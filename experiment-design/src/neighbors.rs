//! Neighbors under the default neighbor rule (design item 11) at the current
//! step of each ordered factor, and how a configuration in a round relates
//! to the incumbent.
//!
//! The finest step of an ordered factor is one position along its value
//! list, since integer and continuous factors are listed at their finest
//! step (design item 31). Steps are counted in those positions.

use std::collections::BTreeMap;

use crate::protocol::{Configuration, FactorId, Protocol, ValueId};

/// Current step per factor that keeps one (`Factor::has_step`), in
/// positions along the factor's value list. Read from the round header
/// (design item 37).
pub type StepMap = BTreeMap<FactorId, usize>;

/// A neighbor and the factor whose move reaches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbor {
    pub factor: FactorId,
    pub config: Configuration,
}

/// Values one move away on `factor` at `step` positions. `step` matters
/// only for factors that keep a step.
fn moves(protocol: &Protocol, factor: FactorId, from: ValueId, step: usize) -> Vec<ValueId> {
    let f = protocol.factor(factor);
    let n = f.values.len();
    if f.is_attributed() {
        // Design item 46: levels that differ in exactly one attribute, by
        // one step for an ordered attribute.
        let here = &f.levels[from.0];
        return (0..n)
            .filter(|&j| {
                let there = &f.levels[j];
                let diffs: Vec<usize> = (0..f.attributes.len())
                    .filter(|&a| here.attributes[a] != there.attributes[a])
                    .collect();
                match diffs.as_slice() {
                    [a] => {
                        !f.attributes[*a].kind.is_ordered()
                            || here.attributes[*a].0.abs_diff(there.attributes[*a].0) == 1
                    }
                    _ => false,
                }
            })
            .map(ValueId)
            .collect();
    }
    if f.kind.is_ordered() {
        // A move past either end of the value list does not exist; the
        // factor moves toward that end only after its step has halved.
        let mut out = Vec::new();
        if from.0 >= step {
            out.push(ValueId(from.0 - step));
        }
        if from.0 + step < n {
            out.push(ValueId(from.0 + step));
        }
        out
    } else {
        (0..n).filter(|&j| j != from.0).map(ValueId).collect()
    }
}

fn step_of(steps: &StepMap, factor: FactorId) -> usize {
    steps.get(&factor).copied().unwrap_or(1)
}

/// Every feasible neighbor of `of` at the given steps, in factor order.
/// A factor absent from `steps` moves by its finest step.
pub fn neighbors_at<'a>(
    protocol: &'a Protocol,
    of: &'a Configuration,
    steps: &'a StepMap,
) -> impl Iterator<Item = Neighbor> + 'a {
    protocol
        .factor_ids()
        .flat_map(move |factor| {
            moves(protocol, factor, of.value(factor), step_of(steps, factor))
                .into_iter()
                .map(move |v| Neighbor {
                    factor,
                    config: of.with(factor, v),
                })
        })
        .filter(move |n| protocol.is_feasible(&n.config))
}

/// Every feasible finest-step neighbor of `of`, in factor order.
pub fn finest_neighbors<'a>(
    protocol: &'a Protocol,
    of: &'a Configuration,
) -> impl Iterator<Item = Neighbor> + 'a {
    static FINEST: StepMap = BTreeMap::new();
    neighbors_at(protocol, of, &FINEST)
}

/// The factor whose one move at the given steps takes `incumbent` to
/// `other`, or `None` when `other` is not such a neighbor.
pub fn relation(
    protocol: &Protocol,
    incumbent: &Configuration,
    other: &Configuration,
    steps: &StepMap,
) -> Option<FactorId> {
    let diffs: Vec<FactorId> = incumbent.differing(other).collect();
    let [factor] = diffs.as_slice() else {
        return None;
    };
    moves(
        protocol,
        *factor,
        incumbent.value(*factor),
        step_of(steps, *factor),
    )
    .contains(&other.value(*factor))
    .then_some(*factor)
}
