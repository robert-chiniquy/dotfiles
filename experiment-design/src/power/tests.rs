#![cfg(test)]

use super::{power, PowerError};
use crate::protocol::Protocol;
use crate::round::Round;

fn protocol(alpha: f64) -> Protocol {
    Protocol::from_yaml(&format!(
        r#"
version: 1
objective: {{kind: number, direction: higher}}
threshold: {{absolute: 8.0}}
delta: 0.5
alpha: {alpha}
budget: {{runs: 60}}
blocks: [saturday, sunday]
factors:
  - {{name: paper, kind: unordered, values: [light, heavy]}}
  - {{name: fold, kind: unordered, values: [dart, glider, delta]}}
baseline: {{paper: light, fold: dart}}
reasonable_moves: all
"#
    ))
    .unwrap()
}

fn noise(p: &Protocol, rows: &[(&str, &str, f64)]) -> Round {
    let mut md = String::from("| run | block | paper | fold | result |\n|---|---|---|---|---|\n");
    for (i, (block, fold, x)) in rows.iter().enumerate() {
        md.push_str(&format!("| n{i} | {block} | light | {fold} | {x} |\n"));
    }
    Round::from_markdown(&md, p).unwrap()
}

#[test]
fn replicates_follow_the_normal_formula() {
    // sd of 10, 11, 12, 13 is sqrt(5/3); z_0.95 + z_0.8 = 2.4865;
    // 2 * (5/3) * 2.4865^2 / 0.25 = 82.4, so 83 per arm.
    let p = protocol(0.05);
    let r = noise(
        &p,
        &[
            ("saturday", "dart", 10.0),
            ("saturday", "dart", 11.0),
            ("sunday", "dart", 12.0),
            ("sunday", "dart", 13.0),
        ],
    );
    let rep = power(&p, &r).unwrap();
    assert_eq!(rep.replicates_formula, 83);
    assert_eq!(rep.replicates, 83);
    // Within each day the spread is 0.5 per pair: pooled sd sqrt(0.5).
    let within = rep.within_block_sd.unwrap();
    assert!((within - 0.5f64.sqrt()).abs() < 1e-12);
    assert!(rep.replicates_paired.unwrap() < rep.replicates);
}

#[test]
fn counting_floor_raises_small_replicate_counts() {
    // A tiny spread asks for 2 runs, but 2 against 2 cannot reach p = 0.05
    // (1/6), so the counting rule needs 3 (1/20).
    let p = protocol(0.05);
    let r = noise(
        &p,
        &[
            ("saturday", "dart", 5.0),
            ("saturday", "dart", 5.01),
            ("sunday", "dart", 5.02),
        ],
    );
    let rep = power(&p, &r).unwrap();
    assert_eq!(rep.replicates_formula, 2);
    assert_eq!(rep.counting_floor, 3);
    assert_eq!(rep.replicates, 3);
}

#[test]
fn alpha_below_005_needs_more_than_three_runs() {
    // Plan risk 19: at alpha 0.01, 4 against 4 gives 1/70 > 0.01; 5 against
    // 5 gives 1/252.
    let p = protocol(0.01);
    let r = noise(&p, &[("saturday", "dart", 5.0), ("sunday", "dart", 5.01)]);
    let rep = power(&p, &r).unwrap();
    assert_eq!(rep.counting_floor, 5);
    assert_eq!(rep.replicates, 5);
    assert!(rep.to_string().contains("plan risk 19"));
}

#[test]
fn noise_rows_must_be_the_baseline() {
    let p = protocol(0.05);
    let r = noise(&p, &[("saturday", "dart", 5.0), ("sunday", "glider", 6.0)]);
    assert_eq!(power(&p, &r), Err(PowerError::NotBaseline("n1".into())));
}

#[test]
fn smallest_delta_grows_as_the_budget_shrinks() {
    // Four arms (baseline, one paper move, two fold moves) and one round,
    // since no factor has a step to halve and every move is reasonable:
    // 60 runs fit 15 per arm, 30 runs fit 7, and fewer runs per arm must
    // raise the detectable delta.
    let rows = [
        ("saturday", "dart", 10.0),
        ("saturday", "dart", 11.0),
        ("sunday", "dart", 12.0),
    ];
    let p = protocol(0.05);
    let wide = power(&p, &noise(&p, &rows)).unwrap();
    let mut small = p.clone();
    small.budget.runs = 30;
    let narrow = power(&small, &noise(&small, &rows)).unwrap();
    assert!(narrow.smallest_delta.unwrap() > wide.smallest_delta.unwrap());
}
