//! The search report (design item 57): each start's verdict, the whole
//! search's verdict (SOLVED, UNSAT, or INCONCLUSIVE, per the verdicts
//! section and decision 62), the unresolved neighbors, the ordered factors
//! sitting at the edge of their allowed values, and the two claims a
//! LOCAL_OPTIMUM or PLATEAU makes, stated separately.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use crate::decide::{DecisionRecord, VerdictKind};
use crate::experiment::ExperimentError;
use crate::neighbors::finest_neighbors;
use crate::protocol::{Configuration, Protocol};
use crate::round::Purpose;
use crate::search::{Search, StartState};
use crate::starts::{same_basin, stop_rule, StopRule};
use crate::stats::{Rule, Test};

mod tests;

/// The two neighbor tests for one neighbor in the verdict round.
#[derive(Debug, Clone, PartialEq)]
pub struct NeighborTests {
    pub neighbor: String,
    pub not_better_p: Option<f64>,
    pub not_better: bool,
    pub worse_p: Option<f64>,
    pub worse: bool,
}

/// What a LOCAL_OPTIMUM or PLATEAU verdict claims, split as the verdicts
/// section requires: an intersection-union claim over the reasonable
/// neighbors, and a sampling bound over the rest of the frontier.
#[derive(Debug, Clone, PartialEq)]
pub struct Claims {
    pub reasonable: Vec<NeighborTests>,
    pub frontier: Vec<NeighborTests>,
    /// Neighbors outside the reasonable moves at the finest step.
    pub frontier_total: usize,
    pub frontier_n: u32,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StartReport {
    pub start: u32,
    pub point: Configuration,
    pub rounds: Vec<u32>,
    pub incumbent: Configuration,
    pub verdict: Option<VerdictKind>,
    /// The round whose decision gave the verdict.
    pub verdict_round: Option<u32>,
    pub rule: Option<Rule>,
    pub not_shown_worse: Vec<String>,
    pub unresolved: Vec<String>,
    /// Ordered factors whose incumbent value is the first or last allowed
    /// value, with that value: the first candidates to widen.
    pub at_edge: Vec<(String, String)>,
    pub claims: Option<Claims>,
    /// DUPLICATE: the earlier start whose incumbent is in the same basin.
    pub duplicate_of: Option<u32>,
    /// SOLVED: the confirmation test's p-value.
    pub confirmation_p: Option<f64>,
}

/// The verdict of the whole search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchVerdict {
    Solved,
    /// No start solved, every non-duplicate start is LOCAL_OPTIMUM or
    /// PLATEAU, and the stop rule holds.
    Unsat,
    /// No start solved and some start ran out of budget.
    Inconclusive,
    /// None of the above yet: a start is still running, or the stop rule
    /// does not hold.
    Open,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub delta: f64,
    pub alpha: f64,
    pub coverage_q: f64,
    pub starts: Vec<StartReport>,
    pub verdict: SearchVerdict,
    pub stop: StopRule,
}

fn verdict_decision<'a>(search: &'a Search, st: &StartState) -> Option<&'a DecisionRecord> {
    st.rounds
        .iter()
        .rev()
        .filter_map(|n| search.entry(*n))
        .filter_map(|e| e.decision.as_ref())
        .find(|d| d.verdict.as_ref().map(|v| v.kind) == st.verdict && st.verdict.is_some())
}

fn at_edge(p: &Protocol, c: &Configuration) -> Vec<(String, String)> {
    p.factor_ids()
        .filter(|f| p.factor(*f).has_step())
        .filter_map(|f| {
            let factor = p.factor(f);
            let v = c.value(f).0;
            (v == 0 || v + 1 == factor.values.len())
                .then(|| (factor.name.clone(), factor.values[v].clone()))
        })
        .collect()
}

fn claims(search: &Search, st: &StartState, d: &DecisionRecord) -> Claims {
    let p = &search.protocol;
    let mut by_neighbor: BTreeMap<String, NeighborTests> = BTreeMap::new();
    for c in &d.comparisons {
        let e = by_neighbor
            .entry(c.neighbor.clone())
            .or_insert_with(|| NeighborTests {
                neighbor: c.neighbor.clone(),
                not_better_p: None,
                not_better: false,
                worse_p: None,
                worse: false,
            });
        match c.test {
            Test::NotBetterByDelta => {
                e.not_better_p = c.p;
                e.not_better = c.reject;
            }
            Test::Worse => {
                e.worse_p = c.p;
                e.worse = c.reject;
            }
            Test::BeatsByDelta | Test::MeetsThreshold => {}
        }
    }
    let reasonable_of = |id: &str| {
        p.parse_config(id).ok().is_some_and(|c| {
            st.incumbent
                .differing(&c)
                .all(|f| p.reasonable_moves.includes(f))
        })
    };
    let (reasonable, frontier): (Vec<NeighborTests>, Vec<NeighborTests>) = by_neighbor
        .into_values()
        .partition(|t| reasonable_of(&t.neighbor));
    let frontier_total = finest_neighbors(p, &st.incumbent)
        .filter(|n| !p.reasonable_moves.includes(n.factor))
        .count();
    let seed = search
        .entry(d.round)
        .and_then(|e| (e.header.purpose == Purpose::FrontierSample).then_some(e.header.seed))
        .flatten();
    Claims {
        reasonable,
        frontier,
        frontier_total,
        frontier_n: p.frontier_sample_size(),
        seed,
    }
}

pub fn report(search: &Search) -> Result<Report, ExperimentError> {
    let p = &search.protocol;
    let states = search.starts()?;
    let ordered: Vec<&StartState> = states.values().collect();
    let mut starts = Vec::new();
    for st in &ordered {
        let d = verdict_decision(search, st);
        let v = d.and_then(|d| d.verdict.as_ref());
        let finished = matches!(
            st.verdict,
            Some(VerdictKind::LocalOptimum | VerdictKind::Plateau)
        );
        starts.push(StartReport {
            start: st.start,
            point: st.point.clone(),
            rounds: st.rounds.clone(),
            incumbent: st.incumbent.clone(),
            verdict: st.verdict,
            verdict_round: d.map(|d| d.round),
            rule: d.map(|d| d.rule),
            not_shown_worse: v.map(|v| v.not_shown_worse.clone()).unwrap_or_default(),
            unresolved: v.map(|v| v.unresolved.clone()).unwrap_or_default(),
            at_edge: if finished || st.verdict == Some(VerdictKind::Solved) {
                at_edge(p, &st.incumbent)
            } else {
                Vec::new()
            },
            claims: match d {
                Some(d) if finished => Some(claims(search, st, d)),
                _ => None,
            },
            duplicate_of: (st.verdict == Some(VerdictKind::Duplicate))
                .then(|| {
                    ordered
                        .iter()
                        .filter(|o| o.start < st.start)
                        .find(|o| same_basin(p, &o.incumbent, &st.incumbent))
                        .map(|o| o.start)
                })
                .flatten(),
            confirmation_p: (st.verdict == Some(VerdictKind::Solved))
                .then(|| d.and_then(|d| d.comparisons.first()).and_then(|c| c.p))
                .flatten(),
        });
    }
    let stop = stop_rule(search, &ordered, p.stop_m);
    let any = |k: VerdictKind| starts.iter().any(|s| s.verdict == Some(k));
    // Decision 62: UNSAT accepts PLATEAU as well as LOCAL_OPTIMUM; both mean
    // no tested neighbor is better by δ.
    let all_basins = !starts.is_empty()
        && starts.iter().all(|s| {
            matches!(
                s.verdict,
                Some(VerdictKind::Duplicate | VerdictKind::LocalOptimum | VerdictKind::Plateau)
            )
        })
        && starts
            .iter()
            .any(|s| s.verdict != Some(VerdictKind::Duplicate));
    let verdict = if any(VerdictKind::Solved) {
        SearchVerdict::Solved
    } else if any(VerdictKind::Budget) {
        SearchVerdict::Inconclusive
    } else if all_basins && stop.holds {
        SearchVerdict::Unsat
    } else {
        SearchVerdict::Open
    };
    Ok(Report {
        delta: p.delta,
        alpha: p.alpha,
        coverage_q: p.coverage_q,
        starts,
        verdict,
        stop,
    })
}

pub fn report_for_dir(dir: &Path) -> Result<(Search, Report), ExperimentError> {
    let search = Search::load(dir)?;
    let r = report(&search)?;
    Ok((search, r))
}

fn p_text(p: Option<f64>) -> String {
    p.map_or_else(|| "no separation".to_string(), |x| format!("p = {x:.4}"))
}

/// The report as markdown; configurations are shown through the protocol.
pub struct ReportDisplay<'a> {
    pub protocol: &'a Protocol,
    pub report: &'a Report,
}

impl fmt::Display for ReportDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = self.protocol;
        let r = self.report;
        writeln!(f, "# Search report\n")?;
        writeln!(
            f,
            "delta {}, alpha {}, coverage q {} (frontier sample n = {}).\n",
            r.delta,
            r.alpha,
            r.coverage_q,
            p.frontier_sample_size()
        )?;
        let rules: BTreeSet<String> = r
            .starts
            .iter()
            .filter_map(|s| s.rule)
            .map(|x| format!("{x:?}"))
            .collect();
        if !rules.is_empty() {
            writeln!(
                f,
                "Rules used in verdict rounds: {}.\n",
                rules.into_iter().collect::<Vec<_>>().join(", ")
            )?;
        }
        for s in &r.starts {
            writeln!(f, "## Start {}\n", s.start)?;
            let rounds: Vec<String> = s.rounds.iter().map(u32::to_string).collect();
            write!(
                f,
                "From {}; rounds {}; incumbent {}. ",
                p.show(&s.point),
                rounds.join(", "),
                p.show(&s.incumbent)
            )?;
            match s.verdict {
                None => writeln!(f, "No verdict yet.")?,
                Some(v) => writeln!(
                    f,
                    "Verdict {v} in round {}.",
                    s.verdict_round.map_or_else(String::new, |n| n.to_string())
                )?,
            }
            if let Some(d) = s.duplicate_of {
                writeln!(
                    f,
                    "\nSame basin as start {d}: the incumbents are within one finest step on every factor."
                )?;
            }
            if s.verdict == Some(VerdictKind::Solved) {
                writeln!(
                    f,
                    "\nThe confirmation round in a fresh block met the threshold ({}).",
                    p_text(s.confirmation_p)
                )?;
            }
            if !s.unresolved.is_empty() {
                writeln!(f, "\nUnresolved: {}.", s.unresolved.join(", "))?;
            }
            if !s.not_shown_worse.is_empty() {
                writeln!(
                    f,
                    "\nNot shown worse (may be chosen on cost): {}.",
                    s.not_shown_worse.join(", ")
                )?;
            }
            if let Some(c) = &s.claims {
                let shown = if s.verdict == Some(VerdictKind::LocalOptimum) {
                    "worse"
                } else {
                    "not better by delta"
                };
                writeln!(
                    f,
                    "\nClaim 1, reasonable neighbors ({}): every one was shown {shown} at alpha {}, each by its own one-sided test; by the intersection-union test (Berger 1982) the claim holds for all of them together at level alpha.",
                    c.reasonable.len(),
                    r.alpha
                )?;
                for t in &c.reasonable {
                    writeln!(
                        f,
                        "- {}: not better by delta {}, worse {}",
                        t.neighbor,
                        p_text(t.not_better_p),
                        p_text(t.worse_p)
                    )?;
                }
                if c.frontier_total == 0 {
                    writeln!(
                        f,
                        "\nClaim 2, rest of the frontier: every neighbor is a reasonable move, so there is no rest of the frontier."
                    )?;
                } else if c.frontier.len() >= c.frontier_total {
                    writeln!(
                        f,
                        "\nClaim 2, rest of the frontier: all {} neighbors outside the reasonable moves were tested, so claim 1 covers them too.",
                        c.frontier_total
                    )?;
                } else {
                    writeln!(
                        f,
                        "\nClaim 2, rest of the frontier: a sampling bound. {} of the {} neighbors outside the reasonable moves were sampled{}. Had a fraction q = {} or more of them been better by delta, a random sample of {} would have missed all of them with probability at most alpha = {}. The bound treats each sampled neighbor's test as correct, so it is approximate when those tests are close calls.",
                        c.frontier.len(),
                        c.frontier_total,
                        c.seed.map_or_else(String::new, |x| format!(" (seed {x})")),
                        r.coverage_q,
                        c.frontier_n,
                        r.alpha
                    )?;
                }
                for t in &c.frontier {
                    writeln!(
                        f,
                        "- {}: not better by delta {}, worse {}",
                        t.neighbor,
                        p_text(t.not_better_p),
                        p_text(t.worse_p)
                    )?;
                }
                writeln!(
                    f,
                    "\nNeither claim covers configurations more than one move from the incumbent."
                )?;
            }
            if !s.at_edge.is_empty() {
                let edge: Vec<String> = s
                    .at_edge
                    .iter()
                    .map(|(n, v)| format!("{n} = {v}"))
                    .collect();
                writeln!(
                    f,
                    "\nAt the edge of their allowed values (first candidates to widen; that is the user's decision): {}.",
                    edge.join(", ")
                )?;
            }
            writeln!(f)?;
        }
        writeln!(f, "## Whole search\n")?;
        let stop =
            format!(
            "Stop rule (last {} starts in known basins, or budget below one more start): {} ({}).",
            p.stop_m,
            if r.stop.holds { "holds" } else { "does not hold" },
            r.stop.reason
        );
        match r.verdict {
            SearchVerdict::Solved => writeln!(f, "SOLVED: a start passed its confirmation round.")?,
            SearchVerdict::Inconclusive => {
                writeln!(
                    f,
                    "INCONCLUSIVE: no start solved and a start ran out of budget; its unresolved neighbors are listed above."
                )?;
            }
            SearchVerdict::Unsat => {
                writeln!(
                    f,
                    "UNSAT: no start solved, every start that is not a duplicate ended at LOCAL_OPTIMUM or PLATEAU, and no new start is warranted. Within the factors and levels given, every basin found has a best configuration below the threshold, and no tested neighbor of any of them is better by delta. Changing the space is the user's decision."
                )?;
            }
            SearchVerdict::Open => {
                writeln!(
                    f,
                    "No whole-search verdict yet: some start has no verdict, or the stop rule does not hold."
                )?;
            }
        }
        writeln!(f, "\n{stop}")
    }
}
