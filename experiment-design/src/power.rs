//! Replicates per arm for the not-better-by-δ test, from phase 1 runs of the
//! baseline (design phase 1, items 14 and 52), and the smallest δ the budget
//! can detect.
//!
//! Method: the normal-approximation sample size for a one-sided two-sample
//! test with equal arms,
//!
//!   n = 2 σ² (z₁₋α + z_power)² / δ²,
//!
//! where the true difference is taken as zero (a neighbor no better than the
//! incumbent) and the test must reject "N − I ≥ δ". Chow, S.-C., Shao, J.,
//! Wang, H., "Sample Size Calculations in Clinical Research", 2nd ed.,
//! Chapman & Hall/CRC, 2008, section 3.2 (two-sample parallel design, test
//! for non-inferiority or superiority). σ is the standard deviation of the
//! phase 1 runs. The same formula solved for δ gives the smallest δ a
//! replicate count can detect.
//!
//! The counting rule (design item 21) decides arms of 5 or fewer runs, and it
//! cannot reject below p = 1 / C(2k, k) with k runs per arm (plan risk 19),
//! so the recommendation is raised to that floor when it applies.

use std::fmt;
use std::path::Path;

use statrs::distribution::{ContinuousCDF, Normal};

use crate::decide::COUNTING_MAX_RUNS;
use crate::experiment::{load_noise, ExperimentError};
use crate::neighbors::{finest_neighbors, neighbors_at};
use crate::protocol::{Objective, Protocol};
use crate::round::{Arm, Outcome, Round};
use crate::stats::separation_p;

mod tests;

/// Power of the not-better-by-δ test that replicates are sized for (design
/// phase 1).
pub const POWER: f64 = 0.8;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PowerError {
    /// The formula is for a measured number; pass rates and preferences
    /// have no phase 1 spread to read.
    #[error("power is computed only for a number objective")]
    NumberOnly,
    #[error("NOISE.md run {0} is not the baseline; phase 1 runs only the baseline")]
    NotBaseline(String),
    #[error("NOISE.md runs not yet run (blank result, no note): {}", .0.join(", "))]
    Pending(Vec<String>),
    /// A standard deviation needs two measured runs.
    #[error("NOISE.md has {0} measured run(s); at least 2 are needed")]
    TooFewRuns(usize),
    /// δ = 0 asks for a difference no replicate count can detect.
    #[error("delta is 0, which no replicate count can detect")]
    DeltaZero,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PowerReport {
    pub delta: f64,
    pub alpha: f64,
    pub runs: usize,
    pub mean: f64,
    /// Standard deviation of all phase 1 runs.
    pub sd: f64,
    /// Pooled within-block standard deviation, when the protocol has blocks
    /// and some block holds two or more phase 1 runs.
    pub within_block_sd: Option<f64>,
    pub z_alpha: f64,
    pub z_power: f64,
    /// The formula's n from `sd`, at least 2.
    pub replicates_formula: u32,
    /// The formula's n from `within_block_sd`.
    pub replicates_paired: Option<u32>,
    /// Smallest k with 1 / C(2k, k) ≤ α.
    pub counting_floor: u32,
    /// Replicates per arm to use: the formula's n, raised to the counting
    /// floor when the counting rule would decide arms of that size.
    pub replicates: u32,
    /// Arms in a first round at the baseline: the baseline and its
    /// reasonable neighbors at the initial steps.
    pub round_arms: usize,
    /// Rounds a start takes with no move at all: one per halving of the
    /// largest initial step, one at the finest step, and one frontier sample
    /// round when the frontier has neighbors outside the reasonable moves.
    pub min_rounds: u32,
    pub budget_runs: u64,
    /// Rounds of `round_arms` × `replicates` runs the budget pays for.
    pub rounds_affordable: u64,
    /// Smallest δ detectable when the budget is spread over `min_rounds`
    /// rounds of `round_arms` arms; `None` when that leaves under 2 runs
    /// per arm.
    pub smallest_delta: Option<f64>,
}

fn sample_size(sd: f64, delta: f64, z: f64) -> u32 {
    let n = 2.0 * sd * sd * z * z / (delta * delta);
    (n.ceil() as u32).max(2)
}

pub fn power(protocol: &Protocol, noise: &Round) -> Result<PowerReport, PowerError> {
    if !matches!(protocol.objective, Objective::Number { .. }) {
        return Err(PowerError::NumberOnly);
    }
    if protocol.delta == 0.0 {
        return Err(PowerError::DeltaZero);
    }
    let pending: Vec<String> = noise.pending().map(|r| r.id.clone()).collect();
    if !pending.is_empty() {
        return Err(PowerError::Pending(pending));
    }
    let mut values: Vec<(Option<&str>, f64)> = Vec::new();
    for run in &noise.runs {
        match &run.arm {
            Arm::Single(c) if *c == protocol.baseline => {}
            _ => return Err(PowerError::NotBaseline(run.id.clone())),
        }
        if let Some(Outcome::Measured(x)) = run.outcome {
            values.push((run.block.as_deref(), x));
        }
    }
    if values.len() < 2 {
        return Err(PowerError::TooFewRuns(values.len()));
    }
    let n = values.len() as f64;
    let mean = values.iter().map(|(_, x)| x).sum::<f64>() / n;
    let sd = (values.iter().map(|(_, x)| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();

    // Pooled within-block spread: what remains once block-to-block drift is
    // removed, which is the spread a comparison paired by block faces.
    let within_block_sd = if protocol.blocks.is_empty() {
        None
    } else {
        let mut ss = 0.0;
        let mut df = 0usize;
        for b in &protocol.blocks {
            let xs: Vec<f64> = values
                .iter()
                .filter(|(blk, _)| *blk == Some(b.as_str()))
                .map(|(_, x)| *x)
                .collect();
            if xs.len() >= 2 {
                let m = xs.iter().sum::<f64>() / xs.len() as f64;
                ss += xs.iter().map(|x| (x - m).powi(2)).sum::<f64>();
                df += xs.len() - 1;
            }
        }
        (df > 0).then(|| (ss / df as f64).sqrt())
    };

    let alpha = protocol.alpha;
    let delta = protocol.delta;
    // Normal::new(0, 1) is always accepted; the match keeps a refusal from
    // panicking if statrs ever changes its checks.
    let z_of = |p: f64| match Normal::new(0.0, 1.0) {
        Ok(d) => d.inverse_cdf(p),
        Err(_) => f64::NAN,
    };
    let z_alpha = z_of(1.0 - alpha);
    let z_power = z_of(POWER);
    let z = z_alpha + z_power;
    let replicates_formula = sample_size(sd, delta, z);
    let replicates_paired = within_block_sd.map(|s| sample_size(s, delta, z));

    let counting_floor = (1..=60u32)
        .find(|&k| separation_p(k as usize, k as usize) <= alpha)
        .unwrap_or(60);
    let max_counting = COUNTING_MAX_RUNS as u32;
    let replicates = if replicates_formula > max_counting {
        replicates_formula
    } else if counting_floor <= max_counting {
        replicates_formula.max(counting_floor)
    } else {
        // The counting rule cannot reach α at any size it decides, so the
        // arms are made large enough for Welch's test instead.
        max_counting + 1
    };

    let steps = protocol.initial_steps();
    let base = &protocol.baseline;
    let round_arms = 1 + neighbors_at(protocol, base, &steps)
        .filter(|n| protocol.reasonable_moves.includes(n.factor))
        .count();
    let halvings = steps
        .values()
        .map(|s| usize::BITS - 1 - s.leading_zeros())
        .max()
        .unwrap_or(0);
    let has_frontier =
        finest_neighbors(protocol, base).any(|n| !protocol.reasonable_moves.includes(n.factor));
    let min_rounds = halvings + 1 + u32::from(has_frontier);

    let budget_runs = protocol.budget.runs;
    let per_round = (round_arms as u64) * u64::from(replicates);
    let rounds_affordable = budget_runs / per_round.max(1);
    let fit = budget_runs / (round_arms as u64 * u64::from(min_rounds)).max(1);
    let smallest_delta = (fit >= 2).then(|| z * sd * (2.0 / fit as f64).sqrt());

    Ok(PowerReport {
        delta,
        alpha,
        runs: values.len(),
        mean,
        sd,
        within_block_sd,
        z_alpha,
        z_power,
        replicates_formula,
        replicates_paired,
        counting_floor,
        replicates,
        round_arms,
        min_rounds,
        budget_runs,
        rounds_affordable,
        smallest_delta,
    })
}

/// `power` on the directory's `PROTOCOL.md` and `NOISE.md`.
pub fn power_for_dir(dir: &Path) -> Result<PowerReport, ExperimentError> {
    let protocol = crate::experiment::load_protocol(dir)?;
    let noise = load_noise(dir, &protocol)?;
    power(&protocol, &noise).map_err(ExperimentError::Power)
}

impl fmt::Display for PowerReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Phase 1: {} baseline run(s), mean {:.4}, standard deviation {:.4}.",
            self.runs, self.mean, self.sd
        )?;
        writeln!(
            f,
            "Replicates per arm for the not-better-by-delta test at alpha {} with {}% power, delta {}: {} (formula n = 2 sd^2 (z_alpha + z_power)^2 / delta^2 = {}, z_alpha {:.4}, z_power {:.4}).",
            self.alpha,
            (POWER * 100.0).round(),
            self.delta,
            self.replicates,
            self.replicates_formula,
            self.z_alpha,
            self.z_power
        )?;
        if self.replicates_formula <= COUNTING_MAX_RUNS as u32 || self.alpha < 0.05 {
            writeln!(
                f,
                "Counting rule floor (plan risk 19): arms of {} or fewer runs are decided by complete separation, whose smallest p with k runs per arm is 1/C(2k, k); reaching alpha {} needs k >= {}.{}",
                COUNTING_MAX_RUNS,
                self.alpha,
                self.counting_floor,
                if self.replicates > self.replicates_formula {
                    format!(" The recommendation is raised from {} to {}.", self.replicates_formula, self.replicates)
                } else {
                    String::new()
                }
            )?;
        }
        if let (Some(s), Some(n)) = (self.within_block_sd, self.replicates_paired) {
            writeln!(
                f,
                "Within-block standard deviation {s:.4} gives {n} per arm for a comparison paired by block. decide does not pair by block yet, so use the unpaired figure above."
            )?;
        }
        writeln!(
            f,
            "Budget: {} runs. A first round has {} arm(s) (baseline and its reasonable neighbors at the initial steps); at {} per arm the budget pays for {} round(s). A start takes at least {} round(s) even with no move.",
            self.budget_runs, self.round_arms, self.replicates, self.rounds_affordable, self.min_rounds
        )?;
        match self.smallest_delta {
            Some(d) => writeln!(
                f,
                "Smallest delta the budget can detect when spread over {} round(s) of {} arm(s): {:.4} (design item 14).",
                self.min_rounds, self.round_arms, d
            )?,
            None => writeln!(
                f,
                "The budget cannot give 2 runs per arm over {} round(s) of {} arm(s); no delta is detectable (design item 14).",
                self.min_rounds, self.round_arms
            )?,
        }
        writeln!(
            f,
            "Assumptions: results are roughly normal with the same spread in every arm as in phase 1; the neighbor's true difference from the incumbent is 0; the figure is for a t-test, and at {} or fewer runs per arm the counting rule decides instead and has less power than this figure; runs in ROUND files are what spend the budget, as decide counts them.",
            COUNTING_MAX_RUNS
        )
    }
}
