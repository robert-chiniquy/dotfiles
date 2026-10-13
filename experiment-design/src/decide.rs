//! The decision for one filled round (design item 56): the comparison rule
//! for the objective, the move decision (item 17), step halving (item 18),
//! the two neighbor tests, and the verdicts LOCAL_OPTIMUM, PLATEAU, and
//! BUDGET (items 27 to 29) with frontier coverage (items 59, 60).
//!
//! Only the given round's runs enter any comparison, so drift between rounds
//! cannot read as an effect (item 17). SOLVED (item 26, phase 5),
//! GUARDRAIL_HALT (item 30), and DUPLICATE (item 58, multi-start) are not
//! decided here.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::neighbors::{finest_neighbors, relation, Relation};
use crate::protocol::{Configuration, Direction, FactorId, Objective, Protocol};
use crate::round::{Arm, Outcome, Round, Side};
use crate::stats::{self, Rule, StatsError, Test};

mod tests;

/// Arms with at most this many runs use the counting rule; larger arms use
/// Welch's test. The design separates "small n" (item 21) from "larger n"
/// (item 22) without a number; the plant example in the skill's
/// `examples.md` switches to the t-test once replicates exceed 5 per arm.
pub const COUNTING_MAX_RUNS: usize = 5;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRecord {
    pub neighbor: String,
    pub test: Test,
    /// `null` when the counting rule found no complete separation.
    pub p: Option<f64>,
    pub reject: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepsWord {
    Unchanged,
}

/// Step sizes after the round: unchanged, or the new step per ordered
/// factor, counted in positions along the factor's value list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Steps {
    Word(StepsWord),
    Set(BTreeMap<String, usize>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerdictKind {
    LocalOptimum,
    Plateau,
    Budget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerdictRecord {
    pub kind: VerdictKind,
    pub incumbent: String,
    pub delta: f64,
    pub alpha: f64,
    pub coverage_q: f64,
    /// Frontier sample size n from q and α (design item 60).
    pub frontier_n: u32,
    /// PLATEAU: neighbors shown not better by δ but not shown worse; any of
    /// them may be chosen on cost (design item 28).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_shown_worse: Vec<String>,
    /// BUDGET: neighbors whose tests did not resolve, or that a verdict
    /// needed and the round did not test (design item 29).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved: Vec<String>,
}

/// The YAML block appended to `DECISIONS.md` for one round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub round: u32,
    pub start: u32,
    pub rule: Rule,
    pub comparisons: Vec<ComparisonRecord>,
    pub dropped_runs: Vec<String>,
    #[serde(rename = "move")]
    pub move_to: Option<String>,
    pub steps: Steps,
    pub verdict: Option<VerdictRecord>,
}

/// A decision and its reading in words.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub record: DecisionRecord,
    pub prose: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DecideError {
    /// Blank result cells with no note: the round is not finished.
    #[error("runs not yet run (blank result, no note): {}", .0.join(", "))]
    Pending(Vec<String>),
    #[error("the incumbent {0} has no results in this round; it is re-measured every round (design item 16)")]
    IncumbentNotRun(String),
    #[error("the round has no neighbor results to compare with the incumbent")]
    NoNeighbors,
    #[error("{0} is not one move from the incumbent under the default neighbor rule")]
    NotANeighbor(String),
    #[error("{0} matches an infeasible combination")]
    Infeasible(String),
    #[error("run {0} is a duel that does not include the incumbent")]
    DuelWithoutIncumbent(String),
    #[error("pass_rate objectives are not supported by this tool yet")]
    PassRateUnsupported,
    #[error(transparent)]
    Stats(#[from] StatsError),
}

/// What decide needs besides the round itself.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    pub protocol: &'a Protocol,
    pub round_number: u32,
    pub start: u32,
    pub incumbent: &'a Configuration,
    /// Runs used by this round and every earlier one.
    pub runs_spent: u64,
}

enum Evidence {
    Values(Vec<f64>),
    Duels { wins: u64, duels: u64 },
}

impl Evidence {
    fn estimate(&self) -> f64 {
        match self {
            Evidence::Values(xs) => xs.iter().sum::<f64>() / xs.len() as f64,
            Evidence::Duels { wins, duels } => *wins as f64 / *duels as f64,
        }
    }
}

struct Tested {
    config: Configuration,
    relation: Relation,
    evidence: Evidence,
}

/// Collect each tested configuration's evidence, oriented so larger is
/// better, and the incumbent's values (empty for a preference objective).
fn gather(cx: &Context<'_>, round: &Round) -> Result<(Vec<f64>, Vec<Tested>), DecideError> {
    let p = cx.protocol;
    let mut by_config: BTreeMap<Configuration, Evidence> = BTreeMap::new();
    let mut incumbent_values = Vec::new();
    match &p.objective {
        Objective::Number { direction, .. } => {
            let sign = match direction {
                Direction::Higher => 1.0,
                Direction::Lower => -1.0,
            };
            for run in &round.runs {
                let (Arm::Single(c), Some(Outcome::Measured(x))) = (&run.arm, run.outcome) else {
                    continue;
                };
                if c == cx.incumbent {
                    incumbent_values.push(sign * x);
                } else if let Evidence::Values(xs) = by_config
                    .entry(c.clone())
                    .or_insert_with(|| Evidence::Values(Vec::new()))
                {
                    xs.push(sign * x);
                }
            }
            if incumbent_values.is_empty() {
                return Err(DecideError::IncumbentNotRun(
                    p.show(cx.incumbent).to_string(),
                ));
            }
        }
        Objective::Preference => {
            for run in &round.runs {
                let (Arm::Duel { left, right }, Some(Outcome::Winner(side))) =
                    (&run.arm, run.outcome)
                else {
                    continue;
                };
                let (other, other_side) = if left == cx.incumbent {
                    (right, Side::Right)
                } else if right == cx.incumbent {
                    (left, Side::Left)
                } else {
                    return Err(DecideError::DuelWithoutIncumbent(run.id.clone()));
                };
                let entry = by_config
                    .entry(other.clone())
                    .or_insert(Evidence::Duels { wins: 0, duels: 0 });
                if let Evidence::Duels { wins, duels } = entry {
                    *duels += 1;
                    if side == other_side {
                        *wins += 1;
                    }
                }
            }
        }
        Objective::PassRate => return Err(DecideError::PassRateUnsupported),
    }
    if by_config.is_empty() {
        return Err(DecideError::NoNeighbors);
    }
    let tested = by_config
        .into_iter()
        .map(|(config, evidence)| {
            if !p.is_feasible(&config) {
                return Err(DecideError::Infeasible(p.show(&config).to_string()));
            }
            match relation(p, cx.incumbent, &config) {
                Relation::NotNeighbor => {
                    Err(DecideError::NotANeighbor(p.show(&config).to_string()))
                }
                relation => Ok(Tested {
                    config,
                    relation,
                    evidence,
                }),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((incumbent_values, tested))
}

/// Run one test. `Ok(None)` is the counting rule's "did not separate".
fn run_test(
    rule: Rule,
    incumbent: &[f64],
    evidence: &Evidence,
    delta: f64,
    test: Test,
) -> Result<Option<f64>, StatsError> {
    match (rule, evidence) {
        (Rule::Counting, Evidence::Values(xs)) => Ok(stats::counting(xs, incumbent, delta, test)),
        (Rule::Welch, Evidence::Values(xs)) => stats::welch(xs, incumbent, delta, test).map(Some),
        (Rule::Binomial, Evidence::Duels { wins, duels }) => {
            stats::binomial(*wins, *duels, delta, test).map(Some)
        }
        // `decide` chooses the rule from the objective, and `gather` builds
        // evidence of the matching kind; a mismatch is a bug here.
        (rule, _) => unreachable!("rule {rule:?} paired with evidence of the other objective kind"),
    }
}

pub fn decide(cx: &Context<'_>, round: &Round) -> Result<Decision, DecideError> {
    let p = cx.protocol;
    let pending: Vec<String> = round.pending().map(|r| r.id.clone()).collect();
    if !pending.is_empty() {
        return Err(DecideError::Pending(pending));
    }
    let dropped: Vec<&crate::round::Run> = round.runs.iter().filter(|r| r.is_dropped()).collect();
    let (incumbent_values, tested) = gather(cx, round)?;

    let rule = match p.objective {
        Objective::Preference => Rule::Binomial,
        _ => {
            let small = incumbent_values.len() <= COUNTING_MAX_RUNS
                && tested.iter().all(|t| match &t.evidence {
                    Evidence::Values(xs) => xs.len() <= COUNTING_MAX_RUNS,
                    Evidence::Duels { .. } => true,
                });
            if small {
                Rule::Counting
            } else {
                Rule::Welch
            }
        }
    };
    let alpha = p.alpha;
    let mut comparisons = Vec::new();
    let mut record = |t: &Tested, test: Test, pv: Option<f64>| -> bool {
        let reject = pv.is_some_and(|x| stats::rejects(x, alpha));
        comparisons.push(ComparisonRecord {
            neighbor: p.show(&t.config).to_string(),
            test,
            p: pv,
            reject,
        });
        reject
    };

    // Move decision (design item 17): uncorrected one-sided test at α; among
    // neighbors that beat the incumbent by δ, the best estimate moves.
    let mut best: Option<&Tested> = None;
    for t in &tested {
        let pv = run_test(
            rule,
            &incumbent_values,
            &t.evidence,
            p.delta,
            Test::BeatsByDelta,
        )?;
        if record(t, Test::BeatsByDelta, pv)
            && best.is_none_or(|b| t.evidence.estimate() > b.evidence.estimate())
        {
            best = Some(t);
        }
    }

    let incumbent_id = p.show(cx.incumbent).to_string();
    let budget_spent = cx.runs_spent >= p.budget.runs;
    let frontier_n = p.frontier_sample_size();
    let verdict_of = |kind, not_shown_worse: Vec<String>, unresolved: Vec<String>| VerdictRecord {
        kind,
        incumbent: incumbent_id.clone(),
        delta: p.delta,
        alpha,
        coverage_q: p.coverage_q,
        frontier_n,
        not_shown_worse,
        unresolved,
    };

    let mut steps = Steps::Word(StepsWord::Unchanged);
    let mut verdict = None;
    let mut coverage_note = String::new();

    if let Some(best) = best {
        if budget_spent {
            let unresolved = tested
                .iter()
                .map(|t| p.show(&t.config).to_string())
                .collect();
            verdict = Some(verdict_of(VerdictKind::Budget, Vec::new(), unresolved));
        }
        let decision = finish(
            cx,
            rule,
            comparisons,
            &dropped,
            Some(p.show(&best.config).to_string()),
            steps,
            verdict,
            String::new(),
        );
        return Ok(decision);
    }

    // No move. A round still at a coarser step on some ordered factor halves
    // that step (design item 18) before any verdict can be considered.
    let coarse: BTreeMap<FactorId, usize> = tested
        .iter()
        .filter_map(|t| match t.relation {
            Relation::Coarse { factor, step } => Some((factor, step)),
            _ => None,
        })
        .fold(BTreeMap::new(), |mut m, (f, s)| {
            let e = m.entry(f).or_insert(s);
            *e = (*e).max(s);
            m
        });
    if !coarse.is_empty() {
        steps = Steps::Set(
            coarse
                .iter()
                .map(|(f, s)| (p.factor(*f).name.clone(), (s / 2).max(1)))
                .collect(),
        );
        if budget_spent {
            let unresolved = tested
                .iter()
                .map(|t| p.show(&t.config).to_string())
                .collect();
            verdict = Some(verdict_of(VerdictKind::Budget, Vec::new(), unresolved));
        }
        return Ok(finish(
            cx,
            rule,
            comparisons,
            &dropped,
            None,
            steps,
            verdict,
            String::new(),
        ));
    }

    // At the finest step with no move: the two neighbor tests on every
    // tested neighbor.
    let mut not_better = Vec::new();
    let mut worse = Vec::new();
    for t in &tested {
        let pv = run_test(
            rule,
            &incumbent_values,
            &t.evidence,
            p.delta,
            Test::NotBetterByDelta,
        )?;
        not_better.push(record(t, Test::NotBetterByDelta, pv));
        let pv = run_test(rule, &incumbent_values, &t.evidence, p.delta, Test::Worse)?;
        worse.push(record(t, Test::Worse, pv));
    }

    // Coverage (design items 59, 60): every reasonable neighbor tested, and
    // at least min(n, remainder) of the rest of the frontier sampled.
    let is_tested = |c: &Configuration| tested.iter().any(|t| &t.config == c);
    let all: Vec<_> = finest_neighbors(p, cx.incumbent).collect();
    let untested_reasonable: Vec<String> = all
        .iter()
        .filter(|n| p.reasonable_moves.includes(n.factor) && !is_tested(&n.config))
        .map(|n| p.show(&n.config).to_string())
        .collect();
    let remainder: Vec<_> = all
        .iter()
        .filter(|n| !p.reasonable_moves.includes(n.factor))
        .collect();
    let sampled = remainder.iter().filter(|n| is_tested(&n.config)).count();
    let required = remainder.len().min(frontier_n as usize);
    let covered = untested_reasonable.is_empty() && sampled >= required;
    if !untested_reasonable.is_empty() {
        let _ = write!(
            coverage_note,
            " No verdict yet: reasonable neighbors untested: {}.",
            untested_reasonable.join(", ")
        );
    } else if sampled < required {
        let _ = write!(
            coverage_note,
            " No verdict yet: the frontier sample has {sampled} of the {required} neighbors needed (n = {frontier_n} for q = {}, α = {alpha}).",
            p.coverage_q
        );
    }

    // Intersection-union test: a claim about all tested neighbors holds when
    // every neighbor's own test rejects at α, and the overall error rate is
    // then at most α with no correction for the number of neighbors.
    // Berger, R. L., "Multiparameter Hypothesis Testing and Acceptance
    // Sampling", Technometrics 24(4), 1982, 295-300.
    // https://doi.org/10.1080/00401706.1982.10487790
    if covered && worse.iter().all(|w| *w) {
        verdict = Some(verdict_of(
            VerdictKind::LocalOptimum,
            Vec::new(),
            Vec::new(),
        ));
    } else if covered && not_better.iter().all(|nb| *nb) {
        let not_shown_worse = tested
            .iter()
            .zip(&worse)
            .filter(|(_, w)| !**w)
            .map(|(t, _)| p.show(&t.config).to_string())
            .collect();
        verdict = Some(verdict_of(
            VerdictKind::Plateau,
            not_shown_worse,
            Vec::new(),
        ));
    } else if budget_spent {
        let mut unresolved: Vec<String> = tested
            .iter()
            .zip(&not_better)
            .filter(|(_, nb)| !**nb)
            .map(|(t, _)| p.show(&t.config).to_string())
            .collect();
        unresolved.extend(untested_reasonable);
        verdict = Some(verdict_of(VerdictKind::Budget, Vec::new(), unresolved));
    }
    Ok(finish(
        cx,
        rule,
        comparisons,
        &dropped,
        None,
        steps,
        verdict,
        coverage_note,
    ))
}

#[allow(clippy::too_many_arguments)]
fn finish(
    cx: &Context<'_>,
    rule: Rule,
    comparisons: Vec<ComparisonRecord>,
    dropped: &[&crate::round::Run],
    move_to: Option<String>,
    steps: Steps,
    verdict: Option<VerdictRecord>,
    note: String,
) -> Decision {
    let record = DecisionRecord {
        round: cx.round_number,
        start: cx.start,
        rule,
        comparisons,
        dropped_runs: dropped.iter().map(|r| r.id.clone()).collect(),
        move_to,
        steps,
        verdict,
    };
    let prose = reading(cx, &record, dropped, &note);
    Decision { record, prose }
}

fn rule_words(rule: Rule) -> &'static str {
    match rule {
        Rule::Counting => "the counting rule (complete separation, exact)",
        Rule::Welch => "Welch's t-test on δ-shifted results",
        Rule::Binomial => "an exact binomial test on duel wins",
    }
}

/// A short reading of the record in words (design item 38).
fn reading(
    cx: &Context<'_>,
    r: &DecisionRecord,
    dropped: &[&crate::round::Run],
    note: &str,
) -> String {
    let p = cx.protocol;
    let mut s = String::new();
    let neighbors = r
        .comparisons
        .iter()
        .filter(|c| c.test == Test::BeatsByDelta)
        .count();
    let _ = write!(
        s,
        "Round {} compared {neighbors} neighbor(s) with the incumbent {} using {}, δ = {}, α = {}.",
        r.round,
        p.show(cx.incumbent),
        rule_words(r.rule),
        p.delta,
        p.alpha
    );
    for run in dropped {
        let _ = write!(
            s,
            " Run {} was dropped (no result; note: {}).",
            run.id, run.notes
        );
    }
    match (&r.move_to, r.rule) {
        (Some(m), Rule::Counting) => {
            let _ = write!(
                s,
                " Every {m} run, after taking δ off, beat every incumbent run, so {m} becomes the incumbent."
            );
        }
        (Some(m), _) => {
            let _ = write!(
                s,
                " {m} beat the incumbent by δ at α and becomes the incumbent."
            );
        }
        (None, _) => s.push_str(" No neighbor beat the incumbent by δ."),
    }
    if let Steps::Set(m) = &r.steps {
        let list: Vec<String> = m.iter().map(|(f, st)| format!("{f} to {st}")).collect();
        let _ = write!(s, " Steps halve: {}.", list.join(", "));
    }
    s.push_str(note);
    if let Some(v) = &r.verdict {
        match v.kind {
            VerdictKind::LocalOptimum => {
                s.push_str(" Verdict LOCAL_OPTIMUM: every tested neighbor was shown worse.")
            }
            VerdictKind::Plateau => {
                let _ = write!(
                    s,
                    " Verdict PLATEAU: every tested neighbor was shown not better by δ; not shown worse: {}.",
                    v.not_shown_worse.join(", ")
                );
            }
            VerdictKind::Budget => {
                let _ = write!(
                    s,
                    " Verdict BUDGET: the run budget is spent; unresolved: {}.",
                    v.unresolved.join(", ")
                );
            }
        }
    }
    s
}
