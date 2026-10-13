//! Multi-start (design items 31 to 35): distance between configurations,
//! the starting points, the skip rule for candidate starts, DUPLICATE, and
//! the rule for when to stop adding starts.
//!
//! The starts already begun are the incumbents in the headers of each
//! start's first round; the next start is chosen against them.

use std::collections::BTreeSet;

use crate::decide::VerdictKind;
use crate::experiment::ExperimentError;
use crate::protocol::{Configuration, FactorId, Protocol, ValueId};
use crate::search::{Search, StartState};
use crate::stats::Test;

mod tests;

/// Searches over more configurations than this are refused: every feasible
/// configuration is listed to pick starts.
pub const MAX_CONFIGURATIONS: usize = 1_000_000;

/// Default m of the stop rule (design item 35).
pub const STOP_AFTER_KNOWN_BASINS: usize = 3;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StartsError {
    #[error("the factors give more than {MAX_CONFIGURATIONS} configurations; starts are picked from a full list")]
    TooManyConfigurations,
}

/// Distance between two configurations (design item 31): summed over
/// factors, the steps apart over the steps in the span for an ordered
/// factor, 0 or 1 for an unordered factor, and the fraction of attributes
/// that differ for an attributed factor.
pub fn distance(p: &Protocol, a: &Configuration, b: &Configuration) -> f64 {
    p.factor_ids()
        .map(|f| {
            let factor = p.factor(f);
            let (x, y) = (a.value(f), b.value(f));
            if factor.is_attributed() {
                let (lx, ly) = (&factor.levels[x.0], &factor.levels[y.0]);
                let differ = lx
                    .attributes
                    .iter()
                    .zip(&ly.attributes)
                    .filter(|(u, v)| u != v)
                    .count();
                differ as f64 / factor.attributes.len() as f64
            } else if factor.kind.is_ordered() {
                match factor.span() {
                    0 => 0.0,
                    span => x.0.abs_diff(y.0) as f64 / span as f64,
                }
            } else {
                f64::from(u8::from(x != y))
            }
        })
        .sum()
}

/// Design item 33: two incumbents within one finest step on every factor
/// are in the same basin. An ordered factor allows one position; an
/// unordered factor has no step, so it must match; an attributed factor
/// must match on unordered attributes and be within one position on
/// ordered ones.
pub fn same_basin(p: &Protocol, a: &Configuration, b: &Configuration) -> bool {
    p.factor_ids().all(|f| {
        let factor = p.factor(f);
        let (x, y) = (a.value(f), b.value(f));
        if factor.is_attributed() {
            let (lx, ly) = (&factor.levels[x.0], &factor.levels[y.0]);
            factor
                .attributes
                .iter()
                .zip(lx.attributes.iter().zip(&ly.attributes))
                .all(|(attr, (u, v))| {
                    if attr.kind.is_ordered() {
                        u.0.abs_diff(v.0) <= 1
                    } else {
                        u == v
                    }
                })
        } else if factor.kind.is_ordered() {
            x.0.abs_diff(y.0) <= 1
        } else {
            x == y
        }
    })
}

/// Every feasible configuration, in configuration order.
pub fn configurations(p: &Protocol) -> Result<Vec<Configuration>, StartsError> {
    let mut total: usize = 1;
    for f in &p.factors {
        total = total
            .checked_mul(f.values.len())
            .filter(|t| *t <= MAX_CONFIGURATIONS)
            .ok_or(StartsError::TooManyConfigurations)?;
    }
    let mut out = Vec::with_capacity(total);
    let mut current = p.baseline.clone();
    let ids: Vec<FactorId> = p.factor_ids().collect();
    let sizes: Vec<usize> = p.factors.iter().map(|f| f.values.len()).collect();
    let mut digits = vec![0usize; ids.len()];
    for _ in 0..total {
        for (i, f) in ids.iter().enumerate() {
            current = current.with(*f, ValueId(digits[i]));
        }
        if p.is_feasible(&current) {
            out.push(current.clone());
        }
        // Increment the last factor fastest so the list is in
        // configuration order.
        for i in (0..digits.len()).rev() {
            digits[i] += 1;
            if digits[i] < sizes[i] {
                break;
            }
            digits[i] = 0;
        }
    }
    Ok(out)
}

/// Why a candidate start was passed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// A local search already ran from it as an incumbent.
    AlreadySearched,
    /// A round showed it worse than that round's incumbent, one move away
    /// (design item 34). An incumbent a neighbor beat by δ is already
    /// covered by `AlreadySearched`.
    BetterNeighborMeasured,
}

/// One pick of the greedy farthest-point selection.
#[derive(Debug, Clone, PartialEq)]
pub struct Pick {
    pub config: Configuration,
    /// Distance to the nearest start chosen before it; 0 for the baseline.
    pub distance: f64,
    /// Candidates farther away that were skipped in its favor.
    pub skipped: Vec<(Configuration, SkipReason)>,
}

/// Configurations a new start should not begin at, from every round.
///
/// Multi-level single linkage skips a sample point when a better point lies
/// within a critical distance, since a local search from it would most
/// likely reach a basin already found. Rinnooy Kan, A. H. G., Timmer, G. T.,
/// "Stochastic global optimization methods part II: Multi level methods",
/// Mathematical Programming 39, 1987, 57-78. Here the critical distance is
/// one move, and "better" is what a round showed under its own comparison
/// rule: comparisons never cross rounds (design item 17), so a candidate is
/// skipped only on evidence from one round.
pub fn skip_set(search: &Search) -> Vec<(Configuration, SkipReason)> {
    let p = &search.protocol;
    let mut skip: Vec<(Configuration, SkipReason)> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut add = |c: Configuration, why| {
        if seen.insert(c.clone()) {
            skip.push((c, why));
        }
    };
    for e in &search.rounds {
        add(e.header.incumbent.clone(), SkipReason::AlreadySearched);
        let Some(d) = &e.decision else { continue };
        for c in &d.comparisons {
            if c.test == Test::Worse && c.reject {
                if let Ok(cfg) = p.parse_config(&c.neighbor) {
                    add(cfg, SkipReason::BetterNeighborMeasured);
                }
            }
        }
    }
    skip
}

/// The next `count` starts after the ones already begun, by greedy
/// farthest-point selection: each pick is the feasible configuration whose
/// distance to the nearest start chosen so far is largest, ties going to
/// the first in configuration order. Gonzalez, T. F., "Clustering to
/// minimize the maximum intercluster distance", Theoretical Computer
/// Science 38, 1985, 293-306; the greedy order is within a factor of 2 of
/// the best maximin set of the same size. With no start begun, the baseline
/// is the first pick.
pub fn next_starts(search: &Search, count: usize) -> Result<Vec<Pick>, ExperimentError> {
    let p = &search.protocol;
    let states = search.starts()?;
    let mut chosen: Vec<Configuration> = states.values().map(|s| s.point.clone()).collect();
    let mut picks = Vec::new();
    if chosen.is_empty() && count > 0 {
        chosen.push(p.baseline.clone());
        picks.push(Pick {
            config: p.baseline.clone(),
            distance: 0.0,
            skipped: Vec::new(),
        });
    }
    let all = configurations(p).map_err(ExperimentError::Starts)?;
    let skip = skip_set(search);
    let skip_reason = |c: &Configuration| skip.iter().find(|(s, _)| s == c).map(|(_, r)| *r);
    let mut nearest: Vec<f64> = all
        .iter()
        .map(|c| {
            chosen
                .iter()
                .map(|s| distance(p, c, s))
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    while picks.len() < count {
        let mut order: Vec<usize> = (0..all.len()).filter(|&i| nearest[i] > 0.0).collect();
        // Stable sort keeps configuration order among equal distances.
        order.sort_by(|&a, &b| nearest[b].total_cmp(&nearest[a]));
        let mut skipped = Vec::new();
        let mut pick = None;
        for i in order {
            match skip_reason(&all[i]) {
                Some(why) => skipped.push((all[i].clone(), why)),
                None => {
                    pick = Some(i);
                    break;
                }
            }
        }
        let Some(i) = pick else { break };
        let c = all[i].clone();
        let d = nearest[i];
        for (j, other) in all.iter().enumerate() {
            nearest[j] = nearest[j].min(distance(p, other, &c));
        }
        picks.push(Pick {
            config: c,
            distance: d,
            skipped,
        });
    }
    Ok(picks)
}

/// Whether to stop adding starts (design item 35), and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopRule {
    pub holds: bool,
    pub reason: String,
}

/// The stop rule: the last `m` starts all ended in known basins
/// (DUPLICATE), or the budget left is less than the runs the cheapest
/// finished start used. A start is finished at SOLVED, LOCAL_OPTIMUM, or
/// PLATEAU.
pub fn stop_rule(search: &Search, states: &[&StartState], m: usize) -> StopRule {
    let p = &search.protocol;
    let last: Vec<&&StartState> = states.iter().rev().take(m).collect();
    if m > 0
        && last.len() == m
        && last
            .iter()
            .all(|s| s.verdict == Some(VerdictKind::Duplicate))
    {
        return StopRule {
            holds: true,
            reason: format!("the last {m} starts all ended in known basins (DUPLICATE)"),
        };
    }
    let spent = search.runs_spent();
    let left = p.budget.runs.saturating_sub(spent);
    let cheapest = states
        .iter()
        .filter(|s| {
            matches!(
                s.verdict,
                Some(VerdictKind::Solved | VerdictKind::LocalOptimum | VerdictKind::Plateau)
            )
        })
        .map(|s| {
            s.rounds
                .iter()
                .filter_map(|n| search.entry(*n))
                .map(|e| e.round.runs.len() as u64)
                .sum::<u64>()
        })
        .min();
    match cheapest {
        Some(c) if left < c => StopRule {
            holds: true,
            reason: format!(
                "the budget has {left} runs left, fewer than the {c} the cheapest finished start used"
            ),
        },
        _ => {
            let dup = last
                .iter()
                .take_while(|s| s.verdict == Some(VerdictKind::Duplicate))
                .count();
            StopRule {
                holds: false,
                reason: format!(
                    "{dup} of the last {m} starts ended in known basins, and the budget has {left} runs left"
                ),
            }
        }
    }
}
