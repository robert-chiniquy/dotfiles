#![cfg(test)]

use super::{report, ReportDisplay, SearchVerdict};
use crate::decide::{DecisionRecord, Steps, StepsWord, VerdictKind, VerdictRecord};
use crate::protocol::Protocol;
use crate::round::{Header, Purpose, Round};
use crate::search::{RoundEntry, Search};
use crate::stats::Rule;

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

/// One decided round per start, each ending in the given verdict.
fn search(verdicts: &[(&str, VerdictKind)]) -> Search {
    let p = grid();
    let rounds = verdicts
        .iter()
        .enumerate()
        .map(|(i, (at, kind))| {
            let n = i as u32 + 1;
            RoundEntry {
                number: n,
                header: Header {
                    round: n,
                    start: n,
                    incumbent: p.parse_config(at).unwrap(),
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
                    round: n,
                    start: n,
                    rule: Rule::Counting,
                    comparisons: Vec::new(),
                    dropped_runs: Vec::new(),
                    move_to: None,
                    steps: Steps::Word(StepsWord::Unchanged),
                    verdict: Some(VerdictRecord {
                        kind: *kind,
                        incumbent: at.to_string(),
                        delta: 0.5,
                        alpha: 0.05,
                        coverage_q: 0.2,
                        frontier_n: 14,
                        not_shown_worse: Vec::new(),
                        unresolved: Vec::new(),
                    }),
                }),
            }
        })
        .collect();
    Search {
        protocol: p,
        rounds,
    }
}

use VerdictKind::{Budget, Duplicate, LocalOptimum, Plateau, Solved};

#[test]
fn unsat_not_returned_while_any_non_duplicate_start_is_budget() {
    let s = search(&[
        ("0/a", LocalOptimum),
        ("4/b", Budget),
        ("0/b", Duplicate),
        ("4/a", Duplicate),
        ("2/b", Duplicate),
    ]);
    assert_eq!(report(&s).unwrap().verdict, SearchVerdict::Inconclusive);
}

#[test]
fn unsat_returned_when_all_non_duplicate_starts_are_local_optimum_or_plateau_and_stop_rule_holds() {
    let s = search(&[
        ("0/a", LocalOptimum),
        ("4/b", Plateau),
        ("0/b", Duplicate),
        ("4/a", Duplicate),
        ("2/b", Duplicate),
    ]);
    let r = report(&s).unwrap();
    assert!(r.stop.holds);
    assert_eq!(r.verdict, SearchVerdict::Unsat);
    let text = ReportDisplay {
        protocol: &s.protocol,
        report: &r,
    }
    .to_string();
    assert!(text.contains("UNSAT"), "{text}");
}

#[test]
fn unsat_waits_for_the_stop_rule() {
    // Same basins, but only two duplicates: a new start is still warranted.
    let s = search(&[
        ("0/a", LocalOptimum),
        ("4/b", Plateau),
        ("0/b", Duplicate),
        ("4/a", Duplicate),
    ]);
    let r = report(&s).unwrap();
    assert!(!r.stop.holds);
    assert_eq!(r.verdict, SearchVerdict::Open);
}

#[test]
fn any_solved_start_solves_the_search() {
    let s = search(&[("0/a", LocalOptimum), ("4/b", Solved)]);
    assert_eq!(report(&s).unwrap().verdict, SearchVerdict::Solved);
}

#[test]
fn local_optimum_at_the_end_of_an_ordered_factor_is_flagged() {
    let s = search(&[("4/b", LocalOptimum)]);
    let r = report(&s).unwrap();
    assert_eq!(
        r.starts[0].at_edge,
        vec![("x".to_string(), "4".to_string())]
    );
}
