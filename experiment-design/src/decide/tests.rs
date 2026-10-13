#![cfg(test)]

use super::{decide, Context, DecideError, Decision, Steps, VerdictKind};
use crate::protocol::Protocol;
use crate::round::{Round, RoundError};
use crate::stats::Test;

/// Twenty strains (frontier, not reasonable) and light hours (reasonable).
/// At q = 0.2, α = 0.05 the frontier sample needs 14 of the 19 strain moves.
fn plants(strains: usize) -> Protocol {
    let values: Vec<String> = (1..=strains).map(|i| format!("s{i:02}")).collect();
    let yaml = format!(
        r#"
version: 1
objective: {{kind: number, direction: higher}}
threshold: {{absolute: 100}}
delta: 0.5
budget: {{runs: 100000}}
factors:
  - {{name: strain, kind: unordered, values: [{}]}}
  - {{name: light, kind: ordered, values: [12, 13, 14, 15, 16, 17, 18]}}
baseline: {{strain: s01, light: 15}}
reasonable_moves: [light]
"#,
        values.join(", ")
    );
    Protocol::from_yaml(&yaml).unwrap()
}

/// A round at the finest light step.
fn round(protocol: &Protocol, arms: &[(&str, &str, f64)]) -> Round {
    round_at(protocol, arms, 1)
}

/// A round with a header giving light the step `light_step`, three runs of
/// the incumbent at 10.0 to 10.2, and three runs of each listed (strain,
/// light) at the given level.
fn round_at(protocol: &Protocol, arms: &[(&str, &str, f64)], light_step: usize) -> Round {
    let mut md = format!(
        "```yaml\nround: 1\nstart: 1\nincumbent: s01/15\npurpose: reasonable_neighbors\nsteps: {{light: {light_step}}}\n```\n\n"
    );
    md.push_str("| run | strain | light | result |\n|---|---|---|---|\n");
    let mut k = 0;
    let mut row = |s: &str, l: &str, x: f64| {
        k += 1;
        md.push_str(&format!("| r{k} | {s} | {l} | {x} |\n"));
    };
    for x in [10.0, 10.1, 10.2] {
        row("s01", "15", x);
    }
    for (s, l, base) in arms {
        for d in [0.0, 0.1, 0.2] {
            row(s, l, base + d);
        }
    }
    Round::from_markdown(&md, protocol).unwrap()
}

fn run(protocol: &Protocol, r: &Round, runs_spent: u64) -> Decision {
    let cx = Context {
        protocol,
        incumbent: &protocol.baseline,
        runs_spent,
    };
    decide(&cx, r).unwrap()
}

#[test]
fn decide_errors_without_step_header() {
    let p = plants(2);
    let arms = [("s02", "15", 5.0), ("s01", "14", 5.0), ("s01", "16", 5.0)];
    let cx = Context {
        protocol: &p,
        incumbent: &p.baseline,
        runs_spent: 0,
    };
    let mut r = round(&p, &arms);
    if let Some(h) = r.header.as_mut() {
        h.steps = None;
    }
    assert!(matches!(decide(&cx, &r), Err(DecideError::StepsMissing)));
    r.header = None;
    assert!(matches!(decide(&cx, &r), Err(DecideError::HeaderMissing)));
}

#[test]
fn step_header_without_ordered_factor_is_rejected() {
    let p = plants(2);
    let md = "```yaml\nround: 1\nstart: 1\nincumbent: s01/15\npurpose: reasonable_neighbors\nsteps: {}\n```\n\n| run | strain | light | result |\n|---|---|---|---|\n| r1 | s01 | 15 | 1 |\n";
    assert!(matches!(
        Round::from_markdown(md, &p),
        Err(RoundError::StepMissing(f)) if f == "light"
    ));
}

#[test]
fn neighbor_off_the_header_step_is_rejected() {
    // Header step 2: light 14 is one position away, not one step.
    let p = plants(2);
    let r = round_at(&p, &[("s02", "15", 5.0), ("s01", "14", 5.0)], 2);
    let cx = Context {
        protocol: &p,
        incumbent: &p.baseline,
        runs_spent: 0,
    };
    assert!(matches!(
        decide(&cx, &r),
        Err(DecideError::NotANeighbor(c)) if c == "s01/14"
    ));
}

fn strains(range: std::ops::RangeInclusive<usize>) -> Vec<(String, &'static str, f64)> {
    range.map(|i| (format!("s{i:02}"), "15", 5.0)).collect()
}

fn arms<'a>(v: &'a [(String, &'static str, f64)]) -> Vec<(&'a str, &'static str, f64)> {
    v.iter().map(|(s, l, x)| (s.as_str(), *l, *x)).collect()
}

#[test]
fn no_optimum_verdict_while_a_reasonable_neighbor_is_untested() {
    // Every tested neighbor is clearly worse, but light 16 was never run.
    let p = plants(20);
    let s = strains(2..=20);
    let mut a = arms(&s);
    a.push(("s01", "14", 5.0));
    let d = run(&p, &round(&p, &a), 0);
    assert_eq!(d.record.verdict, None);
    assert!(d.prose.contains("s01/16"), "{}", d.prose);
}

#[test]
fn no_optimum_verdict_while_frontier_sample_is_below_n() {
    let p = plants(20);
    let s = strains(2..=14); // 13 of the 19 strain moves; n = 14.
    let mut a = arms(&s);
    a.extend([("s01", "14", 5.0), ("s01", "16", 5.0)]);
    assert_eq!(run(&p, &round(&p, &a), 0).record.verdict, None);
}

#[test]
fn local_optimum_once_frontier_sample_reaches_n() {
    let p = plants(20);
    let s = strains(2..=15); // 14 strain moves.
    let mut a = arms(&s);
    a.extend([("s01", "14", 5.0), ("s01", "16", 5.0)]);
    let v = run(&p, &round(&p, &a), 0).record.verdict.unwrap();
    assert_eq!(v.kind, VerdictKind::LocalOptimum);
    assert_eq!(v.frontier_n, 14);
}

#[test]
fn small_frontier_must_be_tested_whole() {
    // Five strains leave four strain moves, fewer than n = 14: all four are
    // needed, not min(n, 4) of some larger set.
    let p = plants(5);
    let s = strains(2..=4);
    let mut a = arms(&s);
    a.extend([("s01", "14", 5.0), ("s01", "16", 5.0)]);
    assert_eq!(run(&p, &round(&p, &a), 0).record.verdict, None);
    let s = strains(2..=5);
    let mut a = arms(&s);
    a.extend([("s01", "14", 5.0), ("s01", "16", 5.0)]);
    let v = run(&p, &round(&p, &a), 0).record.verdict.unwrap();
    assert_eq!(v.kind, VerdictKind::LocalOptimum);
}

#[test]
fn plateau_lists_neighbors_not_shown_worse() {
    let p = plants(2);
    // light 16 overlaps the incumbent (10.0 to 10.2) but stays under +δ.
    let d = run(
        &p,
        &round(
            &p,
            &[("s02", "15", 5.0), ("s01", "14", 5.0), ("s01", "16", 10.1)],
        ),
        0,
    );
    let v = d.record.verdict.unwrap();
    assert_eq!(v.kind, VerdictKind::Plateau);
    assert_eq!(v.not_shown_worse, vec!["s01/16".to_string()]);
}

#[test]
fn coarse_step_halves_instead_of_giving_a_verdict() {
    let p = plants(2);
    let d = run(
        &p,
        &round_at(
            &p,
            &[("s02", "15", 5.0), ("s01", "13", 5.0), ("s01", "17", 5.0)],
            2,
        ),
        0,
    );
    assert_eq!(d.record.verdict, None);
    assert_eq!(
        d.record.steps,
        Steps::Set([("light".to_string(), 1)].into_iter().collect())
    );
    // No neighbor tests are run at a coarse step.
    assert!(d
        .record
        .comparisons
        .iter()
        .all(|c| c.test == Test::BeatsByDelta));
}

#[test]
fn budget_spent_with_unresolved_neighbor_gives_budget() {
    let p = plants(2);
    // light 16 at 10.4 to 10.6 is neither shown not-better by δ nor better.
    let r = round(
        &p,
        &[("s02", "15", 5.0), ("s01", "14", 5.0), ("s01", "16", 10.4)],
    );
    assert_eq!(run(&p, &r, 0).record.verdict, None);
    let v = run(&p, &r, p.budget.runs).record.verdict.unwrap();
    assert_eq!(v.kind, VerdictKind::Budget);
    assert_eq!(v.unresolved, vec!["s01/16".to_string()]);
}

#[test]
fn move_picks_best_estimate_among_neighbors_that_beat_by_delta() {
    let p = plants(2);
    let d = run(
        &p,
        &round(
            &p,
            &[("s02", "15", 11.0), ("s01", "16", 12.0), ("s01", "14", 5.0)],
        ),
        0,
    );
    assert_eq!(d.record.move_to.as_deref(), Some("s01/16"));
    assert_eq!(d.record.verdict, None);
}

#[test]
fn lower_is_better_flips_which_neighbor_beats() {
    let lower = Protocol::from_yaml(PLANTS_LOWER).unwrap();
    // With lower better, 5.0 to 5.2 beats the incumbent's 10.0 to 10.2 by δ.
    let d = run(
        &lower,
        &round(
            &lower,
            &[("s02", "15", 5.0), ("s01", "14", 20.0), ("s01", "16", 20.0)],
        ),
        0,
    );
    assert_eq!(d.record.move_to.as_deref(), Some("s02/15"));
}

const PLANTS_LOWER: &str = r#"
version: 1
objective: {kind: number, direction: lower}
threshold: {absolute: 1}
delta: 0.5
budget: {runs: 100000}
factors:
  - {name: strain, kind: unordered, values: [s01, s02]}
  - {name: light, kind: ordered, values: [12, 13, 14, 15, 16, 17, 18]}
baseline: {strain: s01, light: 15}
reasonable_moves: [light]
"#;
