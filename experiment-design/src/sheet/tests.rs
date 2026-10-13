#![cfg(test)]

use std::collections::BTreeSet;

use super::{next_round, NewRound, RoundRequest};
use crate::decide::{DecisionRecord, Steps, StepsWord};
use crate::neighbors::StepMap;
use crate::protocol::{Configuration, Protocol};
use crate::round::{Arm, Header, Purpose, Round};
use crate::search::{RoundEntry, Search};
use crate::stats::Rule;

/// `strains` unordered strains (moves outside the reasonable set) and light
/// hours 12 to 18 (reasonable), over two benches.
fn plants(strains: usize, batch: u64) -> Protocol {
    let values: Vec<String> = (1..=strains).map(|i| format!("s{i:02}")).collect();
    Protocol::from_yaml(&format!(
        r#"
version: 1
objective: {{kind: number, direction: higher}}
threshold: {{absolute: 100}}
delta: 0.5
budget: {{runs: 100000, batch: {batch}}}
blocks: [bench1, bench2]
factors:
  - {{name: strain, kind: unordered, values: [{}]}}
  - {{name: light, kind: ordered, values: [12, 13, 14, 15, 16, 17, 18]}}
baseline: {{strain: s01, light: 15}}
reasonable_moves: [light]
"#,
        values.join(", ")
    ))
    .unwrap()
}

fn fresh(p: &Protocol) -> Search {
    Search {
        protocol: p.clone(),
        rounds: Vec::new(),
    }
}

fn req(seed: u64) -> RoundRequest {
    RoundRequest {
        replicates: Some(3),
        seed: Some(seed),
        ..RoundRequest::default()
    }
}

/// A search whose one round, at the finest light step, was decided with no
/// move and no verdict.
fn after_no_move_at_finest(p: &Protocol) -> Search {
    let light = p.factor_id("light").unwrap();
    let steps: StepMap = [(light, 1)].into_iter().collect();
    Search {
        protocol: p.clone(),
        rounds: vec![RoundEntry {
            number: 1,
            header: Header {
                round: 1,
                start: 1,
                incumbent: p.baseline.clone(),
                purpose: Purpose::ReasonableNeighbors,
                steps: Some(steps),
                seed: Some(1),
                guardrails: Vec::new(),
            },
            round: Round {
                header: None,
                runs: Vec::new(),
            },
            decision: Some(DecisionRecord {
                round: 1,
                start: 1,
                rule: Rule::Counting,
                comparisons: Vec::new(),
                dropped_runs: Vec::new(),
                move_to: None,
                steps: Steps::Word(StepsWord::Unchanged),
                verdict: None,
            }),
        }],
    }
}

fn configs(new: &NewRound) -> BTreeSet<Configuration> {
    new.runs
        .iter()
        .filter_map(|r| match &r.arm {
            Arm::Single(c) => Some(c.clone()),
            Arm::Duel { .. } => None,
        })
        .collect()
}

#[test]
fn same_seed_gives_same_round() {
    let p = plants(20, 1000);
    let s = after_no_move_at_finest(&p);
    let a = next_round(&s, &req(7), None).unwrap();
    let b = next_round(&s, &req(7), None).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.to_markdown(&p).unwrap(), b.to_markdown(&p).unwrap());
    // A different seed draws a different order or sample.
    let c = next_round(&s, &req(8), None).unwrap();
    assert_ne!(a.runs, c.runs);
}

#[test]
fn incumbent_appears_in_every_block() {
    // One replicate per arm over two benches: the incumbent still runs on
    // both, so every block can pair its arms with the incumbent.
    let p = plants(3, 1000);
    let mut r = req(3);
    r.replicates = Some(1);
    let new = next_round(&fresh(&p), &r, None).unwrap();
    for b in ["bench1", "bench2"] {
        assert!(
            new.runs
                .iter()
                .any(|run| run.block.as_deref() == Some(b)
                    && run.arm == Arm::Single(p.baseline.clone())),
            "no incumbent run in {b}"
        );
    }
}

#[test]
fn first_round_uses_quarter_span_step_and_reasonable_neighbors_only() {
    // Light spans 6 positions; a quarter is 1.5, rounded to 2, so the
    // neighbors of 15 are 13 and 17. Strain moves are not reasonable.
    let p = plants(3, 1000);
    let new = next_round(&fresh(&p), &req(1), None).unwrap();
    let light = p.factor_id("light").unwrap();
    assert_eq!(new.header.steps.as_ref().unwrap()[&light], 2);
    assert_eq!(new.header.purpose, Purpose::ReasonableNeighbors);
    let want: BTreeSet<Configuration> = ["s01/15", "s01/13", "s01/17"]
        .iter()
        .map(|c| p.parse_config(c).unwrap())
        .collect();
    assert_eq!(configs(&new), want);
    assert_eq!(new.runs.len(), 9);
}

#[test]
fn sample_round_re_includes_all_reasonable_neighbors() {
    let p = plants(20, 1000);
    let new = next_round(&after_no_move_at_finest(&p), &req(11), None).unwrap();
    assert_eq!(new.header.purpose, Purpose::FrontierSample);
    assert_eq!(new.header.seed, Some(11));
    let got = configs(&new);
    for c in ["s01/15", "s01/14", "s01/16"] {
        assert!(got.contains(&p.parse_config(c).unwrap()), "{c} missing");
    }
    // 19 strain moves outside the reasonable set; n = 14 of them.
    let sample = new.frontier.unwrap();
    assert_eq!((sample.remainder, sample.n, sample.sampled), (19, 14, 14));
    assert_eq!(got.len(), 3 + 14);
}

#[test]
fn sample_takes_whole_remainder_when_it_has_n_or_fewer() {
    let p = plants(5, 1000);
    let new = next_round(&after_no_move_at_finest(&p), &req(5), None).unwrap();
    let sample = new.frontier.unwrap();
    assert_eq!((sample.remainder, sample.sampled), (4, 4));
    let got = configs(&new);
    for s in ["s02", "s03", "s04", "s05"] {
        assert!(got.contains(&p.parse_config(&format!("{s}/15")).unwrap()));
    }
}

#[test]
fn round_larger_than_batch_is_reported() {
    let p = plants(3, 4);
    let new = next_round(&fresh(&p), &req(1), None).unwrap();
    assert!(
        new.notes.iter().any(|n| n.contains("batch of 4")),
        "{:?}",
        new.notes
    );
}

#[test]
fn written_sheet_reads_back_with_its_header() {
    let p = plants(20, 1000);
    let new = next_round(&after_no_move_at_finest(&p), &req(2), None).unwrap();
    let back = Round::from_markdown(&new.to_markdown(&p).unwrap(), &p).unwrap();
    assert_eq!(back.header.as_ref(), Some(&new.header));
    assert_eq!(back.runs.len(), new.runs.len());
    assert!(back.runs.iter().all(|r| r.is_pending()));
}

/// The search after one round at the finest step where the incumbent
/// s01/15 measured 6, 6.1, 6.2 on bench1 and no neighbor moved.
fn after_round_meeting(p: &Protocol) -> Search {
    let mut s = after_no_move_at_finest(p);
    let md = "| run | block | strain | light | result |\n|---|---|---|---|---|\n| r1 | bench1 | s01 | 15 | 6 |\n| r2 | bench1 | s01 | 15 | 6.1 |\n| r3 | bench1 | s01 | 15 | 6.2 |\n";
    s.rounds[0].round = Round::from_markdown(md, p).unwrap();
    s
}

/// Three strains, threshold `t`.
fn plants_threshold(t: f64) -> Protocol {
    Protocol::from_yaml(&format!(
        r#"
version: 1
objective: {{kind: number, direction: higher}}
threshold: {{absolute: {t}}}
delta: 0.5
budget: {{runs: 100000, batch: 1000}}
blocks: [bench1, bench2]
factors:
  - {{name: strain, kind: unordered, values: [s01, s02, s03]}}
  - {{name: light, kind: ordered, values: [12, 13, 14, 15, 16, 17, 18]}}
baseline: {{strain: s01, light: 15}}
reasonable_moves: [light]
"#
    ))
    .unwrap()
}

#[test]
fn estimate_at_the_threshold_gets_a_confirmation_round_in_a_fresh_block() {
    let p = plants_threshold(6.0);
    let new = next_round(&after_round_meeting(&p), &req(4), None).unwrap();
    assert_eq!(new.header.purpose, Purpose::Confirmation);
    assert!(new
        .runs
        .iter()
        .all(|r| r.block.as_deref() == Some("bench2") && r.arm == Arm::Single(p.baseline.clone())));
    // Below the threshold the search goes on to the frontier sample.
    let p = plants_threshold(7.0);
    let new = next_round(&after_round_meeting(&p), &req(4), None).unwrap();
    assert_eq!(new.header.purpose, Purpose::FrontierSample);
}

#[test]
fn failed_confirmation_resumes_the_search() {
    let p = plants_threshold(6.0);
    let mut s = after_round_meeting(&p);
    let mut confirm = s.rounds[0].clone();
    confirm.number = 2;
    confirm.header.round = 2;
    confirm.header.purpose = Purpose::Confirmation;
    if let Some(d) = confirm.decision.as_mut() {
        d.round = 2;
    }
    s.rounds.push(confirm);
    let new = next_round(&s, &req(4), None).unwrap();
    assert_eq!(new.header.purpose, Purpose::FrontierSample);
    assert_eq!(new.header.round, 3);
}
