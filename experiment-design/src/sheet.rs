//! The next run sheet, `ROUND_<n>.md` (design items 16, 37, 55): the
//! incumbent and every feasible reasonable neighbor at the current steps,
//! replicated, assigned to blocks with the incumbent in every block, and
//! shuffled within each block from a seed the header records. After a round
//! with no move at the finest steps, the sheet adds a seeded sample of n
//! neighbors from the rest of the frontier and measures the reasonable
//! neighbors again (design items 59, 60, and the verdicts section).
//!
//! The sheet is a pure function of the files and the request, so the same
//! seed gives the same sheet.

use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::decide::{Steps, VerdictKind};
use crate::experiment::{round_path, ExperimentError, NOISE_FILE};
use crate::neighbors::{finest_neighbors, neighbors_at, StepMap};
use crate::protocol::{Configuration, Objective, Protocol, Threshold};
use crate::rng::SplitMix64;
use crate::round::{Arm, Header, Outcome, Purpose};
use crate::search::{RoundEntry, Search, StartState};
use crate::starts::{next_starts, stop_rule};

mod tests;

/// What the caller may fix; everything else follows from the files.
#[derive(Debug, Clone, Default)]
pub struct RoundRequest {
    /// The start to continue or begin; default the start of the latest
    /// round, or 1 when there is none.
    pub start: Option<u32>,
    /// Replicates per arm; default the recommendation from `NOISE.md`.
    pub replicates: Option<u32>,
    /// Seed for the run order and the frontier sample; default from the
    /// clock. Recorded in the header either way.
    pub seed: Option<u64>,
    /// Blocks this round uses, in run order; default every protocol block.
    pub blocks: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedRun {
    pub id: String,
    pub block: Option<String>,
    pub arm: Arm,
}

/// The frontier sample of a sample round: `sampled` of the `remainder`
/// neighbors outside the reasonable moves, `n` being the size item 60 asks
/// for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontierSample {
    pub remainder: usize,
    pub n: u32,
    pub sampled: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewRound {
    pub header: Header,
    pub runs: Vec<PlannedRun>,
    pub frontier: Option<FrontierSample>,
    /// Things the person should know before running the sheet: the sheet
    /// exceeding the protocol's batch (plan risk 18) or the budget left.
    pub notes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SheetError {
    #[error(
        "round {0} of this start has no decision yet; decide it before writing the next round"
    )]
    UndecidedRound(u32),
    #[error("start {start} ended with verdict {verdict}; begin the next start with --start")]
    StartEnded { start: u32, verdict: VerdictKind },
    #[error("start {start} cannot begin; the next new start is {expected}")]
    StartNotNext { start: u32, expected: u32 },
    #[error("round {0} has no steps in its header")]
    StepsMissing(u32),
    #[error("replicates per arm are unknown: pass them, or add {NOISE_FILE} with phase 1 runs")]
    ReplicatesUnknown,
    #[error("replicates per arm must be at least 1")]
    ReplicatesZero,
    #[error("block {0:?} is not one of the protocol's blocks")]
    UnknownBlock(String),
    #[error("the incumbent has no feasible neighbor to compare with")]
    NoNeighbors,
    #[error("the budget of {budget} runs is spent ({spent} used)")]
    BudgetSpent { spent: u64, budget: u64 },
    #[error("pass_rate objectives are not supported by this tool yet")]
    PassRateUnsupported,
    #[error("serializing the round header")]
    Header(#[source] serde_saphyr::ser::Error),
    /// Design item 35.
    #[error("no new start: {0}")]
    StopRule(String),
    #[error("no feasible configuration is left to start from")]
    NoStartCandidate,
    /// Phase 5 needs a block the search did not use; the protocol's blocks
    /// are all used.
    #[error("every protocol block has been used; a confirmation round needs a fresh one")]
    NoFreshBlock,
    #[error("block {0:?} was used by an earlier round; a confirmation round needs a fresh one")]
    BlockNotFresh(String),
}

impl From<SheetError> for ExperimentError {
    fn from(e: SheetError) -> Self {
        ExperimentError::Sheet(e)
    }
}

/// What the next round of a start measures.
struct Plan {
    start: u32,
    incumbent: Configuration,
    purpose: Purpose,
    steps: StepMap,
    notes: Vec<String>,
}

/// Whether the incumbent's estimate in `e` meets the threshold, which
/// sends it to a confirmation round (phase 5). False, with a note, when the
/// round cannot say.
fn meets_threshold(
    p: &Protocol,
    e: &RoundEntry,
    inc: &Configuration,
    notes: &mut Vec<String>,
) -> bool {
    if !matches!(p.objective, Objective::Number { .. }) {
        return false;
    }
    let sign = p.orientation();
    let mean_of = |c: &Configuration| {
        let xs: Vec<f64> = e
            .round
            .runs
            .iter()
            .filter(|r| r.arm == Arm::Single(c.clone()))
            .filter_map(|r| match r.outcome {
                Some(Outcome::Measured(x)) => Some(sign * x),
                _ => None,
            })
            .collect();
        (!xs.is_empty()).then(|| xs.iter().sum::<f64>() / xs.len() as f64)
    };
    let Some(m) = mean_of(inc) else {
        return false;
    };
    match p.threshold {
        Threshold::Absolute(t) => m >= sign * t,
        Threshold::MarginOverBaseline(margin) => {
            if *inc == p.baseline {
                return margin <= 0.0;
            }
            match mean_of(&p.baseline) {
                Some(b) => m - b >= margin,
                None => {
                    notes.push(format!(
                        "Round {} did not measure the baseline, so whether {} meets the margin over it is not estimated and no confirmation round is written.",
                        e.number,
                        p.show(inc)
                    ));
                    false
                }
            }
        }
    }
}

fn continue_start(search: &Search, st: &StartState) -> Result<Plan, SheetError> {
    let p = &search.protocol;
    if let Some(n) = st.undecided.first() {
        return Err(SheetError::UndecidedRound(*n));
    }
    let entries: Vec<&RoundEntry> = st.rounds.iter().filter_map(|n| search.entry(*n)).collect();
    let Some(last) = entries.last() else {
        return Err(SheetError::UndecidedRound(0));
    };
    // The search continues from the latest search round; a confirmation
    // round that did not pass leaves the search where it was, and the same
    // estimate is not sent to confirmation twice.
    let after_confirmation = last.header.purpose == Purpose::Confirmation;
    let Some(last_search) = entries
        .iter()
        .rev()
        .find(|e| e.header.purpose != Purpose::Confirmation)
    else {
        return Err(SheetError::UndecidedRound(0));
    };
    let steps = steps_after(search, last_search)?;
    let mut notes = Vec::new();
    let confirmable = matches!(
        st.verdict,
        None | Some(VerdictKind::LocalOptimum | VerdictKind::Plateau)
    );
    if confirmable
        && !after_confirmation
        && meets_threshold(p, last_search, &st.incumbent, &mut notes)
    {
        return Ok(Plan {
            start: st.start,
            incumbent: st.incumbent.clone(),
            purpose: Purpose::Confirmation,
            steps,
            notes,
        });
    }
    if let Some(verdict) = st.verdict {
        return Err(SheetError::StartEnded {
            start: st.start,
            verdict,
        });
    }
    let moved = last_search
        .decision
        .as_ref()
        .is_some_and(|d| d.move_to.is_some());
    let was_finest = last_search
        .header
        .steps
        .as_ref()
        .is_some_and(|m| m.values().all(|s| *s == 1));
    let has_frontier =
        finest_neighbors(p, &st.incumbent).any(|n| !p.reasonable_moves.includes(n.factor));
    // No move at the finest steps: the reasonable neighbors alone cannot
    // give a verdict while the frontier has neighbors outside them.
    let purpose = if !moved && was_finest && has_frontier {
        Purpose::FrontierSample
    } else {
        Purpose::ReasonableNeighbors
    };
    Ok(Plan {
        start: st.start,
        incumbent: st.incumbent.clone(),
        purpose,
        steps,
        notes,
    })
}

fn plan(search: &Search, req: &RoundRequest) -> Result<Plan, ExperimentError> {
    let p = &search.protocol;
    let states = search.starts()?;
    let start = req
        .start
        .or_else(|| search.rounds.last().map(|e| e.header.start))
        .unwrap_or(1);
    if let Some(st) = states.get(&start) {
        return Ok(continue_start(search, st)?);
    }
    let expected = states.keys().max().map_or(1, |m| m + 1);
    if start != expected {
        return Err(SheetError::StartNotNext { start, expected }.into());
    }
    let ordered: Vec<&StartState> = states.values().collect();
    let stop = stop_rule(search, &ordered, p.stop_m);
    if stop.holds {
        return Err(SheetError::StopRule(stop.reason).into());
    }
    let pick = next_starts(search, 1)?
        .into_iter()
        .next()
        .ok_or(SheetError::NoStartCandidate)?;
    let mut notes = Vec::new();
    if !pick.skipped.is_empty() {
        notes.push(format!(
            "Start {start} at {} (distance {:.3} from the nearest start); {} farther candidate(s) skipped because a measured configuration one move away was better or a search already ran from them (design item 34).",
            p.show(&pick.config),
            pick.distance,
            pick.skipped.len()
        ));
    }
    Ok(Plan {
        start,
        incumbent: pick.config,
        purpose: Purpose::ReasonableNeighbors,
        steps: p.initial_steps(),
        notes,
    })
}

/// Steps after an entry's decision: the header's steps with the decision's
/// changes applied.
fn steps_after(search: &Search, e: &RoundEntry) -> Result<StepMap, SheetError> {
    let mut steps = e
        .header
        .steps
        .clone()
        .ok_or(SheetError::StepsMissing(e.number))?;
    if let Some(Steps::Set(changed)) = e.decision.as_ref().map(|d| &d.steps) {
        for (name, s) in changed {
            if let Some(f) = search.protocol.factor_id(name) {
                steps.insert(f, *s);
            }
        }
    }
    Ok(steps)
}

fn clock_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64)
}

/// The next round for the search, not yet written.
pub fn next_round(
    search: &Search,
    req: &RoundRequest,
    default_replicates: Option<u32>,
) -> Result<NewRound, ExperimentError> {
    let p = &search.protocol;
    if matches!(p.objective, Objective::PassRate) {
        return Err(SheetError::PassRateUnsupported.into());
    }
    let spent = search.runs_spent();
    if spent >= p.budget.runs {
        return Err(SheetError::BudgetSpent {
            spent,
            budget: p.budget.runs,
        }
        .into());
    }
    let plan = plan(search, req)?;
    let replicates = req
        .replicates
        .or(default_replicates)
        .ok_or(SheetError::ReplicatesUnknown)?;
    if replicates == 0 {
        return Err(SheetError::ReplicatesZero.into());
    }
    if let Some(b) = req.blocks.iter().flatten().find(|b| !p.blocks.contains(b)) {
        return Err(SheetError::UnknownBlock(b.clone()).into());
    }
    let confirming = plan.purpose == Purpose::Confirmation;
    let blocks = if confirming {
        // Phase 5: a block the search has not used.
        let used = search.blocks_before(u32::MAX);
        match &req.blocks {
            Some(bs) => {
                if let Some(b) = bs.iter().find(|b| used.contains(*b)) {
                    return Err(SheetError::BlockNotFresh(b.clone()).into());
                }
                bs.clone()
            }
            None if p.blocks.is_empty() => Vec::new(),
            None => vec![p
                .blocks
                .iter()
                .find(|b| !used.contains(*b))
                .cloned()
                .ok_or(SheetError::NoFreshBlock)?],
        }
    } else {
        req.blocks.clone().unwrap_or_else(|| p.blocks.clone())
    };
    let seed = req.seed.unwrap_or_else(clock_seed);
    let mut rng = SplitMix64::new(seed);

    let inc = &plan.incumbent;
    let mut arms: Vec<Configuration> = if confirming {
        // The baseline beside the incumbent (phase 5).
        [p.baseline.clone()]
            .into_iter()
            .filter(|b| b != inc)
            .collect()
    } else {
        neighbors_at(p, inc, &plan.steps)
            .filter(|n| p.reasonable_moves.includes(n.factor))
            .map(|n| n.config)
            .collect()
    };
    let mut frontier = None;
    if plan.purpose == Purpose::FrontierSample {
        let remainder: Vec<Configuration> = finest_neighbors(p, inc)
            .filter(|n| !p.reasonable_moves.includes(n.factor))
            .map(|n| n.config)
            .collect();
        let n = p.frontier_sample_size();
        let total = remainder.len();
        let sampled = rng.sample(remainder, n as usize);
        frontier = Some(FrontierSample {
            remainder: total,
            n,
            sampled: sampled.len(),
        });
        arms.extend(sampled);
    }
    if arms.is_empty() && !confirming {
        return Err(SheetError::NoNeighbors.into());
    }

    let runs = lay_out(
        p,
        inc,
        &arms,
        replicates as usize,
        &blocks,
        &mut rng,
        search.next_round_number(),
    );

    let mut notes = plan.notes;
    if let Some(batch) = p.budget.batch {
        if runs.len() as u64 > batch {
            notes.push(format!(
                "This round has {} runs, more than the protocol's batch of {batch} (plan risk 18): {} arm(s) at {replicates} replicate(s).",
                runs.len(),
                arms.len() + 1
            ));
        }
    }
    if spent + runs.len() as u64 > p.budget.runs {
        notes.push(format!(
            "This round has {} runs and the budget has {} left; the decision after it will be BUDGET unless it reaches a verdict.",
            runs.len(),
            p.budget.runs - spent
        ));
    }
    Ok(NewRound {
        header: Header {
            round: search.next_round_number(),
            start: plan.start,
            incumbent: plan.incumbent,
            purpose: plan.purpose,
            steps: Some(plan.steps),
            seed: Some(seed),
            guardrails: p.guardrails.clone(),
        },
        runs,
        frontier,
        notes,
    })
}

/// Rows grouped by block in the order given, shuffled within each block.
/// Every arm gets `replicates` runs spread over the blocks in turn; the
/// incumbent gets at least one run in every block, so each block can pair
/// every arm in it with the incumbent (design item 16). For a preference
/// objective each row is a duel of a neighbor with the incumbent, the side
/// each is shown on drawn at random.
fn lay_out(
    p: &Protocol,
    incumbent: &Configuration,
    arms: &[Configuration],
    replicates: usize,
    blocks: &[String],
    rng: &mut SplitMix64,
    number: u32,
) -> Vec<PlannedRun> {
    let n_blocks = blocks.len().max(1);
    let mut groups: Vec<Vec<Arm>> = vec![Vec::new(); n_blocks];
    match p.objective {
        Objective::Preference => {
            for a in arms {
                for j in 0..replicates {
                    let (left, right) = if rng.below(2) == 0 {
                        (incumbent.clone(), a.clone())
                    } else {
                        (a.clone(), incumbent.clone())
                    };
                    groups[j % n_blocks].push(Arm::Duel { left, right });
                }
            }
        }
        _ => {
            for j in 0..replicates.max(n_blocks) {
                groups[j % n_blocks].push(Arm::Single(incumbent.clone()));
            }
            for a in arms {
                for j in 0..replicates {
                    groups[j % n_blocks].push(Arm::Single(a.clone()));
                }
            }
        }
    }
    let mut runs = Vec::new();
    for (bi, mut group) in groups.into_iter().enumerate() {
        rng.shuffle(&mut group);
        for arm in group {
            runs.push(PlannedRun {
                id: format!("r{number}-{:02}", runs.len() + 1),
                block: blocks.get(bi).cloned(),
                arm,
            });
        }
    }
    runs
}

fn purpose_words(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::ReasonableNeighbors => "the incumbent and every reasonable neighbor",
        Purpose::FrontierSample => {
            "the incumbent, every reasonable neighbor again, and a random sample from the rest of the frontier"
        }
        Purpose::Confirmation => {
            "confirmation: the incumbent and the baseline in a block no earlier round used"
        }
    }
}

impl NewRound {
    pub fn to_markdown(&self, p: &Protocol) -> Result<String, SheetError> {
        let h = &self.header;
        let mut s = String::new();
        let _ = writeln!(s, "# Round {}\n", h.round);
        let _ = write!(
            s,
            "Start {}, incumbent {}. Purpose: {}.",
            h.start,
            p.show(&h.incumbent),
            purpose_words(h.purpose)
        );
        let moving: Vec<String> = h
            .steps
            .iter()
            .flatten()
            .map(|(f, st)| format!("{} {}", p.factor(*f).name, st))
            .collect();
        if !moving.is_empty() {
            let _ = write!(
                s,
                " Current step (positions along each ordered factor's values): {}.",
                moving.join(", ")
            );
        }
        if let Some(fs) = self.frontier {
            let _ = write!(
                s,
                " Frontier sample: {} of the {} neighbors outside the reasonable moves (n = {} for q = {}, alpha = {}), seed {}.",
                fs.sampled,
                fs.remainder,
                fs.n,
                p.coverage_q,
                p.alpha,
                h.seed.map_or_else(String::new, |x| x.to_string())
            );
        }
        if !h.guardrails.is_empty() {
            let _ = write!(s, " Guardrails: {}.", h.guardrails.join("; "));
        }
        s.push_str(
            "\n\nRun the rows in order and fill in result, time, and notes. A run that gives no usable result keeps a blank result and gets a note.\n\n",
        );
        let yaml = h.to_yaml(p).map_err(SheetError::Header)?;
        let _ = write!(s, "```yaml\n{yaml}");
        if !yaml.ends_with('\n') {
            s.push('\n');
        }
        s.push_str("```\n\n");

        let has_block = !p.blocks.is_empty();
        let mut head: Vec<&str> = vec!["run"];
        if has_block {
            head.push("block");
        }
        let preference = matches!(p.objective, Objective::Preference);
        if preference {
            head.extend(["left", "right", "winner"]);
        } else {
            head.extend(p.factors.iter().map(|f| f.name.as_str()));
            head.push("result");
        }
        head.extend(["time", "notes"]);
        let _ = writeln!(s, "| {} |", head.join(" | "));
        let _ = writeln!(s, "|{}", "---|".repeat(head.len()));
        for r in &self.runs {
            let mut cells: Vec<String> = vec![r.id.clone()];
            if has_block {
                cells.push(r.block.clone().unwrap_or_default());
            }
            match &r.arm {
                Arm::Single(c) => {
                    cells.extend(
                        p.factor_ids()
                            .map(|f| p.factor(f).values[c.value(f).0].clone()),
                    );
                }
                Arm::Duel { left, right } => {
                    cells.push(p.show(left).to_string());
                    cells.push(p.show(right).to_string());
                }
            }
            // result or winner, time, notes: left for the person running it.
            let _ = writeln!(s, "| {} | | | |", cells.join(" | "));
        }
        Ok(s)
    }
}

/// Write the next round next to the others. Never overwrites a file.
pub fn write_next_round(
    dir: &Path,
    req: &RoundRequest,
) -> Result<(PathBuf, NewRound), ExperimentError> {
    let search = Search::load(dir)?;
    let default_replicates = if req.replicates.is_none() && dir.join(NOISE_FILE).exists() {
        Some(crate::power::power_for_dir(dir)?.replicates)
    } else {
        None
    };
    let new = next_round(&search, req, default_replicates)?;
    let text = new.to_markdown(&search.protocol)?;
    let path = round_path(dir, new.header.round);
    let io_err = |source| ExperimentError::Io {
        path: path.clone(),
        source,
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(io_err)?;
    file.write_all(text.as_bytes()).map_err(io_err)?;
    Ok((path, new))
}
