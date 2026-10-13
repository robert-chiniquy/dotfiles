//! One-sided tests for one neighbor N against the incumbent I (design items
//! 21, 22, 24; the skill's `comparison-rules.md`).
//!
//! Every number test takes values already oriented so that larger is better;
//! the caller negates values for a lower-is-better objective. Every test is
//! one-sided and rejects when p ≤ α.

use serde::{Deserialize, Serialize};
use statrs::distribution::{Binomial, ContinuousCDF, DiscreteCDF, StudentsT};

mod tests;

/// The three hypotheses tested for a neighbor. Each names the claim that a
/// rejection supports; the null is its complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Test {
    /// N beats I by δ or more (the move test, design item 17). Null: N − I ≤ δ.
    BeatsByDelta,
    /// N is not better than I by δ. Null: N − I ≥ δ.
    NotBetterByDelta,
    /// N is worse than I. Null: N − I ≥ 0.
    Worse,
    /// Phase 5 (verdict 26): the incumbent meets the threshold. Null: it is
    /// below an absolute threshold, or does not beat the baseline by the
    /// margin.
    MeetsThreshold,
}

/// The comparison rule a round used (recorded in the decision log).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Complete separation as an exact permutation test (design item 21).
    Counting,
    /// Welch's t-test on δ-shifted data (design item 22).
    Welch,
    /// Paired t-test on the per-block differences of block means, δ-shifted
    /// (design items 17 and 22), used in place of Welch's test when the runs
    /// of both arms can be paired by block.
    PairedT,
    /// Exact binomial test on duel wins (design item 24).
    Binomial,
    /// One-sample t-test of the incumbent's mean against an absolute
    /// threshold (phase 5, verdict 26).
    OneSampleT,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum StatsError {
    /// Welch's test estimates a variance per arm, which needs two runs.
    #[error("Welch's t-test needs at least 2 runs per arm, got {neighbor} and {incumbent}")]
    TooFewRuns { neighbor: usize, incumbent: usize },
    /// A paired test estimates the spread of per-block differences, which
    /// needs two blocks shared by both arms.
    #[error("paired t-test needs at least 2 blocks with runs of both arms, got {0}")]
    TooFewBlocks(usize),
    #[error("binomial test needs at least one duel")]
    NoDuels,
    /// 0.5 + δ outside (0, 1) leaves no binomial null to test.
    #[error("win-rate null 0.5 + delta = {0} is outside (0, 1)")]
    WinRateOutOfRange(f64),
    /// statrs refused the distribution parameters.
    #[error("distribution parameters rejected: {0}")]
    Distribution(String),
}

pub fn rejects(p: f64, alpha: f64) -> bool {
    p <= alpha
}

/// 1 / C(a + b, a), the chance that every run of one group lands above
/// every run of the other when the labels are exchangeable.
pub fn separation_p(a: usize, b: usize) -> f64 {
    // C(a + b, a) by the multiplicative formula; each partial product is an
    // integer, so the division is exact.
    let mut c: u128 = 1;
    for i in 1..=a.min(b) as u128 {
        c = c * (a.max(b) as u128 + i) / i;
    }
    1.0 / c as f64
}

/// Counting rule (design item 21). Returns p when the δ-shifted groups
/// separate completely in the direction the test claims, and `None` when
/// they do not; the rule then does not reject and records no p.
pub fn counting(neighbor: &[f64], incumbent: &[f64], delta: f64, test: Test) -> Option<f64> {
    if neighbor.is_empty() || incumbent.is_empty() {
        return None;
    }
    let n_min = neighbor.iter().copied().fold(f64::INFINITY, f64::min);
    let n_max = neighbor.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let i_min = incumbent.iter().copied().fold(f64::INFINITY, f64::min);
    let i_max = incumbent.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let separated = match test {
        // Every N run minus δ above every I run. For a margin threshold the
        // caller passes the margin as δ and the baseline as I.
        Test::BeatsByDelta | Test::MeetsThreshold => n_min - delta > i_max,
        // Every N run below every I run plus δ.
        Test::NotBetterByDelta => n_max < i_min + delta,
        // Every N run below every I run.
        Test::Worse => n_max < i_min,
    };
    separated.then(|| separation_p(neighbor.len(), incumbent.len()))
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn sample_variance(xs: &[f64]) -> f64 {
    let m = mean(xs);
    xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (xs.len() - 1) as f64
}

/// Welch's t-test on δ-shifted data (design item 22).
///
/// When both arms have zero spread the statistic is undefined; p is then 0
/// if the shifted difference lies strictly on the side the test claims and 1
/// otherwise, which is the limit of the test as the spread goes to zero.
pub fn welch(
    neighbor: &[f64],
    incumbent: &[f64],
    delta: f64,
    test: Test,
) -> Result<f64, StatsError> {
    let (n1, n2) = (neighbor.len(), incumbent.len());
    if n1 < 2 || n2 < 2 {
        return Err(StatsError::TooFewRuns {
            neighbor: n1,
            incumbent: n2,
        });
    }
    let shift = match test {
        Test::BeatsByDelta | Test::NotBetterByDelta | Test::MeetsThreshold => delta,
        Test::Worse => 0.0,
    };
    let diff = mean(neighbor) - shift - mean(incumbent);
    let a = sample_variance(neighbor) / n1 as f64;
    let b = sample_variance(incumbent) / n2 as f64;
    let se = (a + b).sqrt();
    let upper_tail = matches!(test, Test::BeatsByDelta | Test::MeetsThreshold);
    if se == 0.0 {
        let claimed = if upper_tail { diff > 0.0 } else { diff < 0.0 };
        return Ok(if claimed { 0.0 } else { 1.0 });
    }
    let df = (a + b).powi(2) / (a.powi(2) / (n1 - 1) as f64 + b.powi(2) / (n2 - 1) as f64);
    let t_dist =
        StudentsT::new(0.0, 1.0, df).map_err(|e| StatsError::Distribution(e.to_string()))?;
    let t = diff / se;
    Ok(if upper_tail {
        t_dist.sf(t)
    } else {
        t_dist.cdf(t)
    })
}

/// Paired t-test on `differences`, one per block: the neighbor's block mean
/// minus the incumbent's block mean in the same block (design item 17).
/// Pairing removes the block effect, which a large block-to-block shift
/// would otherwise add to the spread of both arms. The δ shift and the tail
/// follow the test as in `welch`; with zero spread p is the same limit.
pub fn paired_t(differences: &[f64], delta: f64, test: Test) -> Result<f64, StatsError> {
    let n = differences.len();
    if n < 2 {
        return Err(StatsError::TooFewBlocks(n));
    }
    let shift = match test {
        Test::BeatsByDelta | Test::NotBetterByDelta | Test::MeetsThreshold => delta,
        Test::Worse => 0.0,
    };
    let diff = mean(differences) - shift;
    let se = (sample_variance(differences) / n as f64).sqrt();
    let upper_tail = matches!(test, Test::BeatsByDelta | Test::MeetsThreshold);
    if se == 0.0 {
        let claimed = if upper_tail { diff > 0.0 } else { diff < 0.0 };
        return Ok(if claimed { 0.0 } else { 1.0 });
    }
    let t_dist = StudentsT::new(0.0, 1.0, (n - 1) as f64)
        .map_err(|e| StatsError::Distribution(e.to_string()))?;
    let t = diff / se;
    Ok(if upper_tail {
        t_dist.sf(t)
    } else {
        t_dist.cdf(t)
    })
}

/// Exact one-sided binomial test on `wins` of `duels` for the neighbor
/// (design item 24). δ is a win-rate margin.
pub fn binomial(wins: u64, duels: u64, delta: f64, test: Test) -> Result<f64, StatsError> {
    if duels == 0 {
        return Err(StatsError::NoDuels);
    }
    let p0 = match test {
        Test::BeatsByDelta | Test::NotBetterByDelta | Test::MeetsThreshold => 0.5 + delta,
        Test::Worse => 0.5,
    };
    if !(p0 > 0.0 && p0 < 1.0) {
        return Err(StatsError::WinRateOutOfRange(p0));
    }
    let dist = Binomial::new(p0, duels).map_err(|e| StatsError::Distribution(e.to_string()))?;
    Ok(match test {
        // P(X ≥ wins); statrs `sf(x)` is P(X > x).
        Test::BeatsByDelta | Test::MeetsThreshold => {
            if wins == 0 {
                1.0
            } else {
                dist.sf(wins - 1)
            }
        }
        // P(X ≤ wins).
        Test::NotBetterByDelta | Test::Worse => dist.cdf(wins),
    })
}

/// Smallest n with (1 − q)^n ≤ α (design item 60): a random sample of n
/// frontier neighbors misses every one of a fraction q or more that is
/// better by δ with probability at most α.
pub fn frontier_sample_size(q: f64, alpha: f64) -> u32 {
    let mut n = 0u32;
    let mut miss = 1.0f64;
    while miss > alpha {
        n += 1;
        miss *= 1.0 - q;
    }
    n
}

/// One-sided one-sample t-test rejecting "the mean is at or below
/// `threshold`", on values oriented so larger is better (phase 5, absolute
/// threshold). With zero spread, p is 0 when every value is above the
/// threshold and 1 otherwise, the limit as the spread goes to zero.
pub fn one_sample_t(values: &[f64], threshold: f64) -> Result<f64, StatsError> {
    let n = values.len();
    if n < 2 {
        return Err(StatsError::TooFewRuns {
            neighbor: n,
            incumbent: n,
        });
    }
    let diff = mean(values) - threshold;
    let se = (sample_variance(values) / n as f64).sqrt();
    if se == 0.0 {
        return Ok(if diff > 0.0 { 0.0 } else { 1.0 });
    }
    let t_dist = StudentsT::new(0.0, 1.0, (n - 1) as f64)
        .map_err(|e| StatsError::Distribution(e.to_string()))?;
    Ok(t_dist.sf(diff / se))
}
