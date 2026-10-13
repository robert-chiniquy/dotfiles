//! The decision for one filled round (design item 56): the comparison rule
//! for the objective, the move decision (item 17), step halving (item 18),
//! the two neighbor tests, and the verdicts LOCAL_OPTIMUM, PLATEAU, and
//! BUDGET (items 27 to 29) with frontier coverage (items 59, 60).
//!
//! The current steps come from the round header (item 37). A tested
//! configuration must be one move from the incumbent at those steps.
//!
//! A confirmation round (phase 5) is decided by the threshold test and
//! gives SOLVED (item 26) when it rejects. A search round whose incumbent
//! ends within one finest step of an earlier start's incumbent gives
//! DUPLICATE (items 33, 58).
//!
//! Only the given round's runs enter any comparison, so drift between rounds
//! cannot read as an effect (item 17). Past the counting rule's run count,
//! comparisons are paired by block when every arm shares at least two blocks
//! with the arm it is compared against (item 17), and the decision log's
//! `rule` says which test ran. GUARDRAIL_HALT (item 30) is not decided here.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::neighbors::{finest_neighbors, relation, StepMap};
use crate::protocol::{Configuration, Direction, Objective, Protocol, Threshold};
use crate::round::{Arm, Header, Outcome, Purpose, Round, Side};
use crate::starts::same_basin;
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
    /// The incumbent passed the confirmation round (verdict 26).
    Solved,
    LocalOptimum,
    Plateau,
    Budget,
    /// The start's incumbent came within one finest step of an earlier
    /// start's incumbent on every factor (items 33, 58).
    Duplicate,
}

/// The name as the design and the decision log write it.
impl std::fmt::Display for VerdictKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VerdictKind::Solved => "SOLVED",
            VerdictKind::LocalOptimum => "LOCAL_OPTIMUM",
            VerdictKind::Plateau => "PLATEAU",
            VerdictKind::Budget => "BUDGET",
            VerdictKind::Duplicate => "DUPLICATE",
        })
    }
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
    #[error("{0} is not one move from the incumbent at the steps in the round header")]
    NotANeighbor(String),
    /// The round file has no header block, so its start, incumbent, and
    /// steps are unknown.
    #[error("the round file has no header block (round, start, incumbent, purpose, steps)")]
    HeaderMissing,
    /// The header has no `steps` entry. Steps are read from the header
    /// (design item 37), never inferred from the moves in the table.
    #[error("the round header has no steps entry; it must give the current step of every ordered factor (an empty map when there are none)")]
    StepsMissing,
    #[error("{0} matches an infeasible combination")]
    Infeasible(String),
    #[error("run {0} is a duel that does not include the incumbent")]
    DuelWithoutIncumbent(String),
    #[error("pass_rate objectives are not supported by this tool yet")]
    PassRateUnsupported,
    /// A confirmation round runs only the incumbent and the baseline.
    #[error("run {0} in a confirmation round is neither the incumbent nor the baseline")]
    NotInConfirmation(String),
    /// Phase 5 needs a block the search did not use.
    #[error("confirmation run {run} is in block {block:?}, which an earlier round used")]
    BlockNotFresh { run: String, block: String },
    #[error("a confirmation against a margin over the baseline needs baseline runs in the round")]
    BaselineNotRun,
    /// The threshold test for a preference objective is not implemented.
    #[error("confirmation rounds support only a number objective")]
    ConfirmationUnsupported,
    #[error(transparent)]
    Stats(#[from] StatsError),
}

/// What decide needs besides the round itself.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    pub protocol: &'a Protocol,
    /// The incumbent as the decision log derives it; the caller checks it
    /// against the round header.
    pub incumbent: &'a Configuration,
    /// Runs used by this round and every earlier one.
    pub runs_spent: u64,
    /// The current incumbent of every start numbered below this round's,
    /// with its start number, for DUPLICATE (item 33).
    pub earlier_incumbents: &'a [(u32, Configuration)],
    /// Blocks named by runs in earlier rounds; a confirmation round must
    /// use none of them (phase 5).
    pub used_blocks: &'a BTreeSet<String>,
}

/// One arm's results, oriented so larger is better, with the block each
/// came from as an index into the protocol's blocks.
#[derive(Debug, Clone, Default)]
struct Sample {
    all: Vec<f64>,
    by_block: BTreeMap<usize, Vec<f64>>,
}

impl Sample {
    fn push(&mut self, p: &Protocol, block: Option<&String>, x: f64) {
        self.all.push(x);
        if let Some(b) = block.and_then(|b| p.blocks.iter().position(|pb| pb == b)) {
            self.by_block.entry(b).or_default().push(x);
        }
    }

    fn mean(&self) -> f64 {
        self.all.iter().sum::<f64>() / self.all.len() as f64
    }
}

/// Per-block differences of block means, `a` minus `b`, over the blocks
/// where both arms have runs; `None` unless there are at least two such
/// blocks (design item 17: comparisons are paired by block where the domain
/// has blocks).
fn block_differences(a: &Sample, b: &Sample) -> Option<Vec<f64>> {
    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;
    let d: Vec<f64> = a
        .by_block
        .iter()
        .filter_map(|(k, xs)| b.by_block.get(k).map(|ys| mean(xs) - mean(ys)))
        .collect();
    (d.len() >= 2).then_some(d)
}

/// The decision log's record that pairing was possible and not applied:
/// the counting rule is an exact test on the unpaired runs (design item 17
/// asks for pairing; item 21 has no paired form here).
fn pairing_note(rule: Rule, pairable: bool) -> &'static str {
    if rule == Rule::Counting && pairable {
        " The runs could be paired by block, but the counting rule is an unpaired exact test, so pairing was not applied."
    } else {
        ""
    }
}

enum Evidence {
    Values(Sample),
    Duels { wins: u64, duels: u64 },
}

impl Evidence {
    fn estimate(&self) -> f64 {
        match self {
            Evidence::Values(s) => s.mean(),
            Evidence::Duels { wins, duels } => *wins as f64 / *duels as f64,
        }
    }
}

/// The rule for a number comparison of `arms` against `control` (design
/// items 17, 21, 22): counting when every arm has at most
/// `COUNTING_MAX_RUNS` runs; otherwise a paired t-test when every arm can be
/// paired with the control on at least two blocks, and Welch's test when
/// any cannot. The second value says whether every arm could be paired,
/// which the counting rule records as pairing not applied.
fn number_rule<'a>(
    control: &Sample,
    arms: impl Iterator<Item = &'a Sample> + Clone,
) -> (Rule, bool) {
    let pairable = arms
        .clone()
        .all(|s| block_differences(s, control).is_some());
    let small = control.all.len() <= COUNTING_MAX_RUNS
        && arms.clone().all(|s| s.all.len() <= COUNTING_MAX_RUNS);
    let rule = if small {
        Rule::Counting
    } else if pairable {
        Rule::PairedT
    } else {
        Rule::Welch
    };
    (rule, pairable)
}

struct Tested {
    config: Configuration,
    evidence: Evidence,
}

/// Collect each tested configuration's evidence, oriented so larger is
/// better, and the incumbent's values (empty for a preference objective).
fn gather(
    cx: &Context<'_>,
    round: &Round,
    steps: &StepMap,
) -> Result<(Sample, Vec<Tested>), DecideError> {
    let p = cx.protocol;
    let mut by_config: BTreeMap<Configuration, Evidence> = BTreeMap::new();
    let mut incumbent_values = Sample::default();
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
                    incumbent_values.push(p, run.block.as_ref(), sign * x);
                } else if let Evidence::Values(s) = by_config
                    .entry(c.clone())
                    .or_insert_with(|| Evidence::Values(Sample::default()))
                {
                    s.push(p, run.block.as_ref(), sign * x);
                }
            }
            if incumbent_values.all.is_empty() {
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
    let mut tested = Vec::new();
    for (config, evidence) in by_config {
        if !p.is_feasible(&config) {
            return Err(DecideError::Infeasible(p.show(&config).to_string()));
        }
        match relation(p, cx.incumbent, &config, steps) {
            Some(_) => tested.push(Tested { config, evidence }),
            // The baseline is the control of every round (design item 10),
            // not a neighbor: it is never a move candidate unless it is also
            // one move from the incumbent.
            None if config == p.baseline => {}
            None => return Err(DecideError::NotANeighbor(p.show(&config).to_string())),
        }
    }
    if tested.is_empty() {
        return Err(DecideError::NoNeighbors);
    }
    Ok((incumbent_values, tested))
}

/// Run one test. `Ok(None)` is the counting rule's "did not separate".
fn run_test(
    rule: Rule,
    incumbent: &Sample,
    evidence: &Evidence,
    delta: f64,
    test: Test,
) -> Result<Option<f64>, StatsError> {
    match (rule, evidence) {
        (Rule::Counting, Evidence::Values(s)) => {
            Ok(stats::counting(&s.all, &incumbent.all, delta, test))
        }
        (Rule::Welch, Evidence::Values(s)) => {
            stats::welch(&s.all, &incumbent.all, delta, test).map(Some)
        }
        (Rule::PairedT, Evidence::Values(s)) => {
            let d = block_differences(s, incumbent).unwrap_or_default();
            stats::paired_t(&d, delta, test).map(Some)
        }
        (Rule::Binomial, Evidence::Duels { wins, duels }) => {
            stats::binomial(*wins, *duels, delta, test).map(Some)
        }
        // `decide` chooses the rule from the objective, and `gather` builds
        // evidence of the matching kind; a mismatch is a bug here.
        (rule, _) => unreachable!("rule {rule:?} paired with evidence of the other objective kind"),
    }
}

pub fn decide(cx: &Context<'_>, round: &Round) -> Result<Decision, DecideError> {
    let header = round.header.as_ref().ok_or(DecideError::HeaderMissing)?;
    let pending: Vec<String> = round.pending().map(|r| r.id.clone()).collect();
    if !pending.is_empty() {
        return Err(DecideError::Pending(pending));
    }
    match header.purpose {
        Purpose::Confirmation => confirm(cx, header, round),
        Purpose::ReasonableNeighbors | Purpose::FrontierSample => search_round(cx, header, round),
    }
}

fn search_round(cx: &Context<'_>, header: &Header, round: &Round) -> Result<Decision, DecideError> {
    let p = cx.protocol;
    let header_steps = header.steps.as_ref().ok_or(DecideError::StepsMissing)?;
    let dropped: Vec<&crate::round::Run> = round.runs.iter().filter(|r| r.is_dropped()).collect();
    let (incumbent_values, tested) = gather(cx, round, header_steps)?;

    let (rule, pairable) = match p.objective {
        Objective::Preference => (Rule::Binomial, false),
        _ => number_rule(
            &incumbent_values,
            tested.iter().filter_map(|t| match &t.evidence {
                Evidence::Values(s) => Some(s),
                Evidence::Duels { .. } => None,
            }),
        ),
    };
    let pairing_note = pairing_note(rule, pairable);
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
    let mut coverage_note = pairing_note.to_string();

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
            header,
            rule,
            comparisons,
            &dropped,
            Some(&best.config),
            steps,
            verdict,
            pairing_note.to_string(),
        );
        return Ok(decision);
    }

    // No move. Every ordered factor whose step in the header is above its
    // finest step halves (design item 18), and no verdict is considered
    // until all of them are at the finest step.
    let coarse: BTreeMap<String, usize> = header_steps
        .iter()
        .filter(|(_, s)| **s > 1)
        .map(|(f, s)| (p.factor(*f).name.clone(), s / 2))
        .collect();
    if !coarse.is_empty() {
        steps = Steps::Set(coarse);
        if budget_spent {
            let unresolved = tested
                .iter()
                .map(|t| p.show(&t.config).to_string())
                .collect();
            verdict = Some(verdict_of(VerdictKind::Budget, Vec::new(), unresolved));
        }
        return Ok(finish(
            cx,
            header,
            rule,
            comparisons,
            &dropped,
            None,
            steps,
            verdict,
            pairing_note.to_string(),
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
    // at least min(n, remainder) of the rest of the frontier sampled. The
    // baseline runs in every round whether drawn or not, so it is not part
    // of the sample and does not count toward it.
    let is_tested = |c: &Configuration| tested.iter().any(|t| &t.config == c);
    let all: Vec<_> = finest_neighbors(p, cx.incumbent).collect();
    let untested_reasonable: Vec<String> = all
        .iter()
        .filter(|n| p.reasonable_moves.includes(n.factor) && !is_tested(&n.config))
        .map(|n| p.show(&n.config).to_string())
        .collect();
    let remainder: Vec<_> = all
        .iter()
        .filter(|n| !p.reasonable_moves.includes(n.factor) && n.config != p.baseline)
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
        header,
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
    header: &Header,
    rule: Rule,
    comparisons: Vec<ComparisonRecord>,
    dropped: &[&crate::round::Run],
    move_to: Option<&Configuration>,
    steps: Steps,
    mut verdict: Option<VerdictRecord>,
    mut note: String,
) -> Decision {
    let p = cx.protocol;
    // DUPLICATE (items 33, 58): the incumbent after this round is in a
    // basin an earlier start already reached, so this start stops. It
    // replaces any other verdict of the round.
    let now = move_to.unwrap_or(cx.incumbent);
    if let Some((earlier, other)) = cx
        .earlier_incumbents
        .iter()
        .find(|(_, c)| same_basin(p, now, c))
    {
        let _ = write!(
            note,
            " {} is within one finest step on every factor of start {earlier}'s incumbent {}: the same basin.",
            p.show(now),
            p.show(other)
        );
        verdict = Some(VerdictRecord {
            kind: VerdictKind::Duplicate,
            incumbent: p.show(now).to_string(),
            delta: p.delta,
            alpha: p.alpha,
            coverage_q: p.coverage_q,
            frontier_n: p.frontier_sample_size(),
            not_shown_worse: Vec::new(),
            unresolved: Vec::new(),
        });
    }
    let record = DecisionRecord {
        round: header.round,
        start: header.start,
        rule,
        comparisons,
        dropped_runs: dropped.iter().map(|r| r.id.clone()).collect(),
        move_to: move_to.map(|c| p.show(c).to_string()),
        steps,
        verdict,
    };
    let prose = reading(cx, &record, dropped, &note);
    Decision { record, prose }
}

/// Phase 5 (verdict 26): the incumbent, and the baseline beside it, in a
/// block no earlier round used. For an absolute threshold a one-sample
/// t-test rejects "the incumbent is below the threshold"; for a margin over
/// the baseline the round's comparison rule rejects "the incumbent does
/// not beat the baseline by the margin". SOLVED when it rejects at α.
fn confirm(cx: &Context<'_>, header: &Header, round: &Round) -> Result<Decision, DecideError> {
    let p = cx.protocol;
    if !matches!(p.objective, Objective::Number { .. }) {
        return Err(DecideError::ConfirmationUnsupported);
    }
    let sign = p.orientation();
    let mut inc = Sample::default();
    let mut base = Sample::default();
    for run in &round.runs {
        let Arm::Single(c) = &run.arm else {
            return Err(DecideError::NotInConfirmation(run.id.clone()));
        };
        if let Some(b) = &run.block {
            if cx.used_blocks.contains(b) {
                return Err(DecideError::BlockNotFresh {
                    run: run.id.clone(),
                    block: b.clone(),
                });
            }
        }
        let Some(Outcome::Measured(x)) = run.outcome else {
            continue;
        };
        if c == cx.incumbent {
            inc.push(p, run.block.as_ref(), sign * x);
        } else if *c == p.baseline {
            base.push(p, run.block.as_ref(), sign * x);
        } else {
            return Err(DecideError::NotInConfirmation(run.id.clone()));
        }
    }
    if inc.all.is_empty() {
        return Err(DecideError::IncumbentNotRun(
            p.show(cx.incumbent).to_string(),
        ));
    }
    let (rule, pv, against, pairing_note) = match p.threshold {
        Threshold::Absolute(t) => (
            Rule::OneSampleT,
            Some(stats::one_sample_t(&inc.all, sign * t)?),
            format!("the threshold {t}"),
            "",
        ),
        Threshold::MarginOverBaseline(m) => {
            if base.all.is_empty() {
                return Err(DecideError::BaselineNotRun);
            }
            let (rule, pairable) = number_rule(&base, std::iter::once(&inc));
            let note = pairing_note(rule, pairable);
            let ev = Evidence::Values(inc.clone());
            (
                rule,
                run_test(rule, &base, &ev, m, Test::MeetsThreshold)?,
                format!("the baseline by the margin {m}"),
                note,
            )
        }
    };
    let reject = pv.is_some_and(|x| stats::rejects(x, p.alpha));
    let incumbent = p.show(cx.incumbent).to_string();
    let verdict = reject.then(|| VerdictRecord {
        kind: VerdictKind::Solved,
        incumbent: incumbent.clone(),
        delta: p.delta,
        alpha: p.alpha,
        coverage_q: p.coverage_q,
        frontier_n: p.frontier_sample_size(),
        not_shown_worse: Vec::new(),
        unresolved: Vec::new(),
    });
    let dropped: Vec<&crate::round::Run> = round.runs.iter().filter(|r| r.is_dropped()).collect();
    let record = DecisionRecord {
        round: header.round,
        start: header.start,
        rule,
        comparisons: vec![ComparisonRecord {
            neighbor: incumbent.clone(),
            test: Test::MeetsThreshold,
            p: pv,
            reject,
        }],
        dropped_runs: dropped.iter().map(|r| r.id.clone()).collect(),
        move_to: None,
        steps: Steps::Word(StepsWord::Unchanged),
        verdict,
    };
    let mut prose = format!(
        "Round {} is the confirmation of {incumbent} in a fresh block, tested against {against} with {}, α = {}.",
        header.round,
        rule_words(rule),
        p.alpha
    );
    prose.push_str(pairing_note);
    for run in &dropped {
        let _ = write!(
            prose,
            " Run {} was dropped (no result; note: {}).",
            run.id, run.notes
        );
    }
    let p_text = pv.map_or_else(
        || "no complete separation".to_string(),
        |x| format!("p = {x}"),
    );
    if reject {
        let _ = write!(prose, " It rejects ({p_text}). Verdict SOLVED.");
    } else {
        let _ = write!(
            prose,
            " It does not reject ({p_text}); the start is not solved and continues from its last search round."
        );
    }
    Ok(Decision { record, prose })
}

fn rule_words(rule: Rule) -> &'static str {
    match rule {
        Rule::Counting => "the counting rule (complete separation, exact)",
        Rule::Welch => "Welch's t-test on δ-shifted results",
        Rule::PairedT => "a paired t-test on δ-shifted per-block differences of block means",
        Rule::Binomial => "an exact binomial test on duel wins",
        Rule::OneSampleT => "a one-sample t-test against the threshold",
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
            VerdictKind::Duplicate => s.push_str(" Verdict DUPLICATE: this start stops."),
            VerdictKind::Solved => s.push_str(" Verdict SOLVED."),
        }
    }
    s
}
