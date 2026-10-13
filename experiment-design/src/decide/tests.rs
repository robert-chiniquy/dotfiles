#![cfg(test)]

use std::collections::BTreeSet;

use super::{decide, Context, DecideError, Decision, Steps, VerdictKind};
use crate::protocol::Protocol;
use crate::round::{Round, RoundError};
use crate::stats::{self, Rule, Test};

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
    round_from(protocol, ("s01", "15"), arms, light_step)
}

/// `round_at` with the incumbent (strain, light) given.
fn round_from(
    protocol: &Protocol,
    (inc_strain, inc_light): (&str, &str),
    arms: &[(&str, &str, f64)],
    light_step: usize,
) -> Round {
    let mut md = format!(
        "```yaml\nround: 1\nstart: 1\nincumbent: {inc_strain}/{inc_light}\npurpose: reasonable_neighbors\nsteps: {{light: {light_step}}}\n```\n\n"
    );
    md.push_str("| run | strain | light | result |\n|---|---|---|---|\n");
    let mut k = 0;
    let mut row = |s: &str, l: &str, x: f64| {
        k += 1;
        md.push_str(&format!("| r{k} | {s} | {l} | {x} |\n"));
    };
    for x in [10.0, 10.1, 10.2] {
        row(inc_strain, inc_light, x);
    }
    for (s, l, base) in arms {
        for d in [0.0, 0.1, 0.2] {
            row(s, l, base + d);
        }
    }
    Round::from_markdown(&md, protocol).unwrap()
}

static NO_BLOCKS: BTreeSet<String> = BTreeSet::new();

/// Context at the baseline with no earlier start and no used block.
fn context(protocol: &Protocol, runs_spent: u64) -> Context<'_> {
    Context {
        protocol,
        incumbent: &protocol.baseline,
        runs_spent,
        earlier_incumbents: &[],
        used_blocks: &NO_BLOCKS,
    }
}

fn run(protocol: &Protocol, r: &Round, runs_spent: u64) -> Decision {
    decide(&context(protocol, runs_spent), r).unwrap()
}

#[test]
fn decide_errors_without_step_header() {
    let p = plants(2);
    let arms = [("s02", "15", 5.0), ("s01", "14", 5.0), ("s01", "16", 5.0)];
    let cx = context(&p, 0);
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
    assert!(matches!(
        decide(&context(&p, 0), &r),
        Err(DecideError::NotANeighbor(c)) if c == "s01/14"
    ));
}

#[test]
fn duplicate_when_incumbents_are_within_one_finest_step_on_every_factor() {
    let p = plants(2);
    let arms = [("s02", "15", 5.0), ("s01", "14", 5.0), ("s01", "16", 5.0)];
    // An earlier start sits at s01/16: one light position away, same strain.
    let near = [(1, p.parse_config("s01/16").unwrap())];
    let cx = Context {
        earlier_incumbents: &near,
        ..context(&p, 0)
    };
    let v = decide(&cx, &round(&p, &arms))
        .unwrap()
        .record
        .verdict
        .unwrap();
    assert_eq!(v.kind, VerdictKind::Duplicate);
    assert_eq!(v.incumbent, "s01/15");
    // Two positions away, or another strain, is a different basin.
    for other in ["s01/17", "s02/15"] {
        let far = [(1, p.parse_config(other).unwrap())];
        let cx = Context {
            earlier_incumbents: &far,
            ..context(&p, 0)
        };
        let d = decide(&cx, &round(&p, &arms)).unwrap();
        assert_ne!(
            d.record.verdict.map(|v| v.kind),
            Some(VerdictKind::Duplicate),
            "{other}"
        );
    }
}

/// A confirmation round of the incumbent s01/15 with the given results.
fn confirmation(p: &Protocol, results: &[f64]) -> Round {
    let mut md = String::from(
        "```yaml\nround: 2\nstart: 1\nincumbent: s01/15\npurpose: confirmation\n```\n\n| run | strain | light | result |\n|---|---|---|---|\n",
    );
    for (i, x) in results.iter().enumerate() {
        md.push_str(&format!("| c{i} | s01 | 15 | {x} |\n"));
    }
    Round::from_markdown(&md, p).unwrap()
}

#[test]
fn solved_requires_the_confirmation_test_to_reject() {
    // Threshold 100. Mean 102 with sd 1 over 3 runs: t = 3.46, df 2,
    // p = 0.037, rejects at 0.05.
    let p = plants(2);
    let d = run(&p, &confirmation(&p, &[101.0, 102.0, 103.0]), 0);
    let c = &d.record.comparisons[0];
    assert_eq!((c.test, c.reject), (Test::MeetsThreshold, true));
    assert_eq!(d.record.verdict.unwrap().kind, VerdictKind::Solved);
    // Mean 101 with sd 2: the estimate is above the threshold, but the
    // test does not reject, so no SOLVED.
    let d = run(&p, &confirmation(&p, &[99.0, 101.0, 103.0]), 0);
    assert!(!d.record.comparisons[0].reject);
    assert_eq!(d.record.verdict, None);
}

#[test]
fn confirmation_in_a_used_block_is_rejected() {
    let p = Protocol::from_yaml(&PLANTS_LOWER.replace(
        "budget: {runs: 100000}",
        "budget: {runs: 100000}\nblocks: [b1, b2]",
    ))
    .unwrap();
    let md = "```yaml\nround: 2\nstart: 1\nincumbent: s01/15\npurpose: confirmation\n```\n\n| run | block | strain | light | result |\n|---|---|---|---|---|\n| c1 | b1 | s01 | 15 | 0.1 |\n| c2 | b1 | s01 | 15 | 0.2 |\n";
    let r = Round::from_markdown(md, &p).unwrap();
    let used: BTreeSet<String> = ["b1".to_string()].into_iter().collect();
    let cx = Context {
        used_blocks: &used,
        ..context(&p, 0)
    };
    assert!(matches!(
        decide(&cx, &r),
        Err(DecideError::BlockNotFresh { block, .. }) if block == "b1"
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
fn baseline_control_is_not_a_move_candidate_unless_a_neighbor() {
    // Incumbent s02/12: the baseline s01/15 differs on both factors, so it
    // is the round's control, not a neighbor, even though it measures far
    // above the incumbent.
    let p = plants(2);
    let inc = p.parse_config("s02/12").unwrap();
    let cx = Context {
        incumbent: &inc,
        ..context(&p, 0)
    };
    let r = round_from(
        &p,
        ("s02", "12"),
        &[("s02", "13", 5.0), ("s01", "15", 20.0)],
        1,
    );
    let d = decide(&cx, &r).unwrap();
    assert_eq!(d.record.move_to, None);
    assert!(
        d.record.comparisons.iter().all(|c| c.neighbor != "s01/15"),
        "{:?}",
        d.record.comparisons
    );
    // From s01/16 the baseline is one light step away: a neighbor, and it
    // moves.
    let inc = p.parse_config("s01/16").unwrap();
    let cx = Context {
        incumbent: &inc,
        ..context(&p, 0)
    };
    let r = round_from(
        &p,
        ("s01", "16"),
        &[("s01", "17", 5.0), ("s01", "15", 20.0)],
        1,
    );
    assert_eq!(
        decide(&cx, &r).unwrap().record.move_to.as_deref(),
        Some("s01/15")
    );
}

#[test]
fn baseline_control_does_not_count_toward_frontier_sample() {
    // Incumbent s02/15 among twenty strains: its 19 strain moves include
    // s01/15, the baseline, which runs as the control whether drawn or not.
    // The sample is drawn from the other 18 and needs n = 14 of them.
    let p = plants(20);
    let inc = p.parse_config("s02/15").unwrap();
    let cx = Context {
        incumbent: &inc,
        ..context(&p, 0)
    };
    let tested = |drawn: std::ops::RangeInclusive<usize>| {
        let s = strains(drawn);
        let mut a = arms(&s);
        a.extend([("s02", "14", 5.0), ("s02", "16", 5.0), ("s01", "15", 5.0)]);
        decide(&cx, &round_from(&p, ("s02", "15"), &a, 1))
            .unwrap()
            .record
            .verdict
    };
    // 13 drawn plus the baseline is 14 tested, but only 13 drawn.
    assert_eq!(tested(3..=15), None);
    assert_eq!(
        tested(3..=16).map(|v| v.kind),
        Some(VerdictKind::LocalOptimum)
    );
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

/// Two strains and light, over three benches.
const PLANTS_BLOCKED: &str = r#"
version: 1
objective: {kind: number, direction: higher}
threshold: {absolute: 100}
delta: 0.5
budget: {runs: 100000}
blocks: [b1, b2, b3]
factors:
  - {name: strain, kind: unordered, values: [s01, s02]}
  - {name: light, kind: ordered, values: [12, 13, 14, 15, 16, 17, 18]}
baseline: {strain: s01, light: 15}
reasonable_moves: [light]
"#;

/// A round of incumbent s01/15 at the finest step from (block, light,
/// result) rows, strain s01 throughout.
fn blocked_round(p: &Protocol, rows: &[(&str, &str, f64)]) -> Round {
    let mut md = String::from(
        "```yaml\nround: 1\nstart: 1\nincumbent: s01/15\npurpose: reasonable_neighbors\nsteps: {light: 1}\n```\n\n| run | block | strain | light | result |\n|---|---|---|---|---|\n",
    );
    for (i, (b, l, x)) in rows.iter().enumerate() {
        md.push_str(&format!("| r{i} | {b} | s01 | {l} | {x} |\n"));
    }
    Round::from_markdown(&md, p).unwrap()
}

#[test]
fn paired_test_finds_a_difference_a_block_effect_hides_from_welch() {
    // Synthetic data. Bench b2 reads about 20 above b1 and b3 about 40
    // above b1. Within every bench, light 16 is about 1.0 above the
    // incumbent at light 15, more than δ = 0.5; light 14 is about 5 below.
    let p = Protocol::from_yaml(PLANTS_BLOCKED).unwrap();
    let rows = [
        ("b1", "15", 10.0),
        ("b1", "15", 10.2),
        ("b2", "15", 30.1),
        ("b2", "15", 29.9),
        ("b3", "15", 50.0),
        ("b3", "15", 50.2),
        ("b1", "16", 11.1),
        ("b1", "16", 11.1),
        ("b2", "16", 31.0),
        ("b2", "16", 30.8),
        ("b3", "16", 51.2),
        ("b3", "16", 51.0),
        ("b1", "14", 5.0),
        ("b1", "14", 5.2),
        ("b2", "14", 25.1),
        ("b2", "14", 24.9),
        ("b3", "14", 45.0),
        ("b3", "14", 45.2),
    ];
    let d = run(&p, &blocked_round(&p, &rows), 0);
    assert_eq!(d.record.rule, Rule::PairedT);
    assert_eq!(d.record.move_to.as_deref(), Some("s01/16"));
    // Unpaired, the 40-point spread across benches swamps the 1.0 gain.
    let pick = |l: &str| -> Vec<f64> {
        rows.iter()
            .filter(|(_, x, _)| *x == l)
            .map(|(_, _, v)| *v)
            .collect()
    };
    let unpaired = stats::welch(&pick("16"), &pick("15"), 0.5, Test::BeatsByDelta).unwrap();
    assert!(!stats::rejects(unpaired, p.alpha), "{unpaired}");
}

#[test]
fn unpairable_round_falls_back_to_welch() {
    // Light 14 ran only on b1, so not every arm shares two benches with the
    // incumbent.
    let p = Protocol::from_yaml(PLANTS_BLOCKED).unwrap();
    let mut rows: Vec<(&str, &str, f64)> = Vec::new();
    for (b, base) in [("b1", 10.0), ("b2", 10.0), ("b3", 10.0)] {
        rows.extend([(b, "15", base), (b, "15", base + 0.2)]);
        rows.extend([(b, "16", base - 3.0), (b, "16", base - 2.8)]);
    }
    for x in [5.0, 5.1, 5.2, 5.3, 5.4, 5.5] {
        rows.push(("b1", "14", x));
    }
    let d = run(&p, &blocked_round(&p, &rows), 0);
    assert_eq!(d.record.rule, Rule::Welch);
}

#[test]
fn counting_rule_records_that_pairing_was_not_applied() {
    let p = Protocol::from_yaml(PLANTS_BLOCKED).unwrap();
    let rows = [
        ("b1", "15", 10.0),
        ("b2", "15", 10.1),
        ("b3", "15", 10.2),
        ("b1", "16", 5.0),
        ("b2", "16", 5.1),
        ("b3", "16", 5.2),
        ("b1", "14", 5.0),
        ("b2", "14", 5.1),
        ("b3", "14", 5.2),
    ];
    let d = run(&p, &blocked_round(&p, &rows), 0);
    assert_eq!(d.record.rule, Rule::Counting);
    assert!(d.prose.contains("pairing was not applied"), "{}", d.prose);
}
