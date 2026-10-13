#![cfg(test)]

use super::{Arm, Outcome, Round, RoundError, Side};
use crate::protocol::Protocol;

const NUMBER: &str = r#"
version: 1
objective: {kind: number, direction: higher}
threshold: {absolute: 8.0}
delta: 0.5
budget: {runs: 60}
blocks: [saturday, sunday]
factors:
  - {name: paper, kind: unordered, values: [light, heavy]}
  - {name: fold, kind: unordered, values: [dart, glider]}
baseline: {paper: light, fold: dart}
reasonable_moves: all
"#;

#[test]
fn columns_are_found_by_name_not_position() {
    // Factor columns swapped and result placed first; rows still read right.
    let p = Protocol::from_yaml(NUMBER).unwrap();
    let md = "| result | notes | fold | run | paper | block |\n|---|---|---|---|---|---|\n| 6.1 | | glider | r1 | heavy | sunday |\n";
    let round = Round::from_markdown(md, &p).unwrap();
    let run = &round.runs[0];
    assert_eq!(run.id, "r1");
    assert_eq!(run.block.as_deref(), Some("sunday"));
    assert_eq!(
        run.arm,
        Arm::Single(p.parse_config("heavy/glider").unwrap())
    );
    assert_eq!(run.outcome, Some(Outcome::Measured(6.1)));
}

#[test]
fn blank_result_is_pending_and_blank_with_note_is_dropped() {
    let p = Protocol::from_yaml(NUMBER).unwrap();
    let md = "| run | block | paper | fold | result | notes |\n|---|---|---|---|---|---|\n| r1 | saturday | light | dart | | |\n| r2 | saturday | light | dart | | gust of wind |\n";
    let round = Round::from_markdown(md, &p).unwrap();
    assert!(round.runs[0].is_pending());
    assert!(round.runs[1].is_dropped());
    assert_eq!(round.pending().count(), 1);
}

#[test]
fn missing_factor_column_is_named() {
    let p = Protocol::from_yaml(NUMBER).unwrap();
    let md = "| run | block | paper | result |\n|---|---|---|---|\n| r1 | saturday | light | 5 |\n";
    assert!(
        matches!(Round::from_markdown(md, &p), Err(RoundError::MissingColumn(c)) if c == "fold")
    );
}

#[test]
fn rejects_non_numeric_result_and_unknown_block() {
    let p = Protocol::from_yaml(NUMBER).unwrap();
    let bad = "| run | block | paper | fold | result |\n|---|---|---|---|---|\n| r1 | saturday | light | dart | far |\n";
    assert!(matches!(
        Round::from_markdown(bad, &p),
        Err(RoundError::NotANumber { .. })
    ));
    let block = "| run | block | paper | fold | result |\n|---|---|---|---|---|\n| r1 | monday | light | dart | 5 |\n";
    assert!(matches!(
        Round::from_markdown(block, &p),
        Err(RoundError::UnknownBlock { .. })
    ));
}

#[test]
fn preference_rows_read_left_right_and_winner() {
    let p = Protocol::from_yaml(
        &NUMBER
            .replace("{kind: number, direction: higher}", "{kind: preference}")
            .replace("delta: 0.5", "delta: 0.1"),
    )
    .unwrap();
    let md = "| run | block | left | right | winner |\n|---|---|---|---|---|\n| d1 | saturday | light/dart | heavy/dart | heavy/dart |\n| d2 | saturday | light/dart | heavy/dart | light/glider |\n";
    let err = Round::from_markdown(md, &p).unwrap_err();
    assert!(matches!(err, RoundError::WinnerNotInDuel { ref run, .. } if run == "d2"));
    let round =
        Round::from_markdown(&md.replace("| light/glider |", "| light/dart |"), &p).unwrap();
    assert_eq!(round.runs[0].outcome, Some(Outcome::Winner(Side::Right)));
    assert_eq!(round.runs[1].outcome, Some(Outcome::Winner(Side::Left)));
}
