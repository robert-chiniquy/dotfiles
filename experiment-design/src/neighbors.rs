//! Neighbors under the default neighbor rule (design item 11) at the finest
//! step, and how a configuration in a round relates to the incumbent.
//!
//! The finest step of an ordered factor is one position along its value
//! list, since integer and continuous factors are listed at their finest
//! step (design item 31).

use crate::protocol::{Configuration, FactorId, Protocol, ValueId};

/// A neighbor and the factor whose move reaches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbor {
    pub factor: FactorId,
    pub config: Configuration,
}

/// Values one finest-step move away on `factor`.
fn finest_moves(protocol: &Protocol, factor: FactorId, from: ValueId) -> Vec<ValueId> {
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
        let mut out = Vec::new();
        if from.0 > 0 {
            out.push(ValueId(from.0 - 1));
        }
        if from.0 + 1 < n {
            out.push(ValueId(from.0 + 1));
        }
        out
    } else {
        (0..n).filter(|&j| j != from.0).map(ValueId).collect()
    }
}

/// Every feasible finest-step neighbor of `of`, in factor order.
pub fn finest_neighbors<'a>(
    protocol: &'a Protocol,
    of: &'a Configuration,
) -> impl Iterator<Item = Neighbor> + 'a {
    protocol
        .factor_ids()
        .flat_map(move |factor| {
            finest_moves(protocol, factor, of.value(factor))
                .into_iter()
                .map(move |v| Neighbor {
                    factor,
                    config: of.with(factor, v),
                })
        })
        .filter(move |n| protocol.is_feasible(&n.config))
}

/// How a configuration tested in a round relates to the incumbent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// One move at the finest step.
    Finest { factor: FactorId },
    /// One move on an ordered factor by `step` positions, `step` > 1: the
    /// round is still at a coarser step on that factor (design item 15).
    Coarse { factor: FactorId, step: usize },
    /// Not one move away under the default rule.
    NotNeighbor,
}

pub fn relation(protocol: &Protocol, incumbent: &Configuration, other: &Configuration) -> Relation {
    let diffs: Vec<FactorId> = incumbent.differing(other).collect();
    let [factor] = diffs.as_slice() else {
        return Relation::NotNeighbor;
    };
    let factor = *factor;
    let (from, to) = (incumbent.value(factor), other.value(factor));
    if finest_moves(protocol, factor, from).contains(&to) {
        return Relation::Finest { factor };
    }
    let f = protocol.factor(factor);
    if f.kind.is_ordered() && !f.is_attributed() {
        Relation::Coarse {
            factor,
            step: from.0.abs_diff(to.0),
        }
    } else {
        Relation::NotNeighbor
    }
}
