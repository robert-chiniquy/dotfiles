#![cfg(test)]

use super::{distance, next_starts, same_basin, skip_set, stop_rule, SkipReason};
use crate::decide::{
    ComparisonRecord, DecisionRecord, Steps, StepsWord, VerdictKind, VerdictRecord,
};
use crate::protocol::{Configuration, Protocol};
use crate::round::{Header, Purpose, Round};
use crate::search::{RoundEntry, Search, StartState};
use crate::stats::{Rule, Test};

/// x ordered over five values, y unordered over two; baseline 0/a.
fn grid() -> Protocol {
    Protocol::from_yaml(
        r#"
version: 1
objective: {kind: number, direction: higher}
threshold: {absolute: 100}
delta: 0.5
budget: {runs: 1000}
factors:
  - {name: x, kind: ordered, values: [0, 1, 2, 3, 4]}
  - {name: y, kind: unordered, values: [a, b]}
baseline: {x: 0, y: a}
reasonable_moves: all
"#,
    )
    .unwrap()
}

fn cfg(p: &Protocol, id: &str) -> Configuration {
    p.parse_config(id).unwrap()
}

/// A decided round of `start` at `incumbent` with the given comparisons and
/// verdict.
fn entry(
    p: &Protocol,
    number: u32,
    start: u32,
    incumbent: &str,
    comparisons: Vec<ComparisonRecord>,
    verdict: Option<VerdictKind>,
) -> RoundEntry {
    RoundEntry {
        number,
        header: Header {
            round: number,
            start,
            incumbent: cfg(p, incumbent),
            purpose: Purpose::ReasonableNeighbors,
            steps: Some(Default::default()),
            seed: None,
            guardrails: Vec::new(),
        },
        round: Round {
            header: None,
            runs: Vec::new(),
        },
        decision: Some(DecisionRecord {
            round: number,
            start,
            rule: Rule::Counting,
            comparisons,
            dropped_runs: Vec::new(),
            move_to: None,
            steps: Steps::Word(StepsWord::Unchanged),
            verdict: verdict.map(|kind| VerdictRecord {
                kind,
                incumbent: incumbent.to_string(),
                delta: 0.5,
                alpha: 0.05,
                coverage_q: 0.2,
                frontier_n: 14,
                not_shown_worse: Vec::new(),
                unresolved: Vec::new(),
            }),
        }),
    }
}

fn search(p: &Protocol, rounds: Vec<RoundEntry>) -> Search {
    Search {
        protocol: p.clone(),
        rounds,
    }
}

#[test]
fn baseline_is_always_the_first_start() {
    let p = grid();
    let picks = next_starts(&search(&p, Vec::new()), 3).unwrap();
    assert_eq!(picks[0].config, p.baseline);
    assert_eq!(picks.len(), 3);
}

#[test]
fn farthest_point_picks_the_max_distance_configuration_on_a_small_grid() {
    let p = grid();
    let picks = next_starts(&search(&p, Vec::new()), 3).unwrap();
    // 4/b is 4/4 + 1 = 2 from 0/a, the largest distance on the grid.
    assert_eq!(picks[1].config, cfg(&p, "4/b"));
    assert_eq!(picks[1].distance, 2.0);
    // Next, 0/b and 4/a are each 1 from the nearest start; 0/b comes first
    // in configuration order.
    assert_eq!(picks[2].config, cfg(&p, "0/b"));
    assert_eq!(picks[2].distance, 1.0);
}

#[test]
fn distance_sums_ordered_fraction_and_unordered_mismatch() {
    let p = grid();
    assert_eq!(distance(&p, &cfg(&p, "1/a"), &cfg(&p, "3/b")), 0.5 + 1.0);
    assert_eq!(distance(&p, &cfg(&p, "2/a"), &cfg(&p, "2/a")), 0.0);
}

#[test]
fn candidate_shown_worse_next_to_a_better_one_is_skipped() {
    // Start 1 found 4/b worse than its incumbent 3/b; 4/b would otherwise be
    // the farthest point from the baseline.
    let p = grid();
    let worse = ComparisonRecord {
        neighbor: "4/b".into(),
        test: Test::Worse,
        p: Some(0.05),
        reject: true,
    };
    let s = search(&p, vec![entry(&p, 1, 1, "0/a", vec![worse], None)]);
    assert!(skip_set(&s).contains(&(cfg(&p, "4/b"), SkipReason::BetterNeighborMeasured)));
    let pick = &next_starts(&s, 1).unwrap()[0];
    assert_ne!(pick.config, cfg(&p, "4/b"));
    assert!(pick
        .skipped
        .contains(&(cfg(&p, "4/b"), SkipReason::BetterNeighborMeasured)));
}

#[test]
fn same_basin_needs_every_factor_within_one_finest_step() {
    let p = grid();
    assert!(same_basin(&p, &cfg(&p, "2/a"), &cfg(&p, "3/a")));
    assert!(!same_basin(&p, &cfg(&p, "2/a"), &cfg(&p, "4/a")));
    assert!(!same_basin(&p, &cfg(&p, "2/a"), &cfg(&p, "2/b")));
}

fn states(s: &Search) -> Vec<StartState> {
    s.starts().unwrap().into_values().collect()
}

#[test]
fn stop_rule_holds_after_three_duplicates_in_a_row() {
    let p = grid();
    let mut rounds = vec![entry(
        &p,
        1,
        1,
        "0/a",
        vec![],
        Some(VerdictKind::LocalOptimum),
    )];
    for (n, at) in [(2, "4/b"), (3, "0/b"), (4, "4/a")] {
        rounds.push(entry(&p, n, n, at, vec![], Some(VerdictKind::Duplicate)));
    }
    let s = search(&p, rounds.clone());
    let st = states(&s);
    let refs: Vec<&StartState> = st.iter().collect();
    assert!(stop_rule(&s, &refs, 3).holds);
    // Two duplicates are not enough.
    let s = search(&p, rounds[..3].to_vec());
    let st = states(&s);
    let refs: Vec<&StartState> = st.iter().collect();
    assert!(!stop_rule(&s, &refs, 3).holds);
}
