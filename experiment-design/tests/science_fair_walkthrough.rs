//! End to end on the science fair walkthrough in the skill's
//! `references/examples.md`: the tool must reach the same decisions a person
//! reaches by hand. Fixture data are synthetic.

use std::fs;
use std::path::{Path, PathBuf};

use experiment_design::{
    check, decide_round, distance, load_decisions, power_for_dir, write_next_round,
    ExperimentError, Purpose, RoundRequest, Rule, Search, SheetError, Test, VerdictKind,
};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/science_fair");

/// A fresh copy of the fixture under the target directory, so decisions are
/// appended to the copy and never to the checked-in fixture.
fn copy_fixture(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for entry in fs::read_dir(FIXTURE).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), dir.join(entry.file_name())).unwrap();
    }
    dir
}

#[test]
fn fixture_passes_check() {
    let report = check(Path::new(FIXTURE)).unwrap();
    assert_eq!(report.rounds.len(), 2);
    assert!(report.rounds.iter().all(|r| r.runs == 15 && r.pending == 0));
}

#[test]
fn round_one_moves_to_glider_and_round_two_is_plateau() {
    let dir = copy_fixture("walkthrough");

    let d1 = decide_round(&dir, 1).unwrap().record;
    assert_eq!(d1.rule, Rule::Counting);
    assert_eq!(d1.move_to.as_deref(), Some("light/glider/no"));
    assert_eq!(d1.verdict, None);
    let glider = d1
        .comparisons
        .iter()
        .find(|c| c.neighbor == "light/glider/no" && c.test == Test::BeatsByDelta)
        .unwrap();
    assert_eq!(glider.p, Some(0.05));
    assert!(glider.reject);
    // Taking δ off light/dart/yes leaves 5.0, below the incumbent's 5.4.
    let clip = d1
        .comparisons
        .iter()
        .find(|c| c.neighbor == "light/dart/yes")
        .unwrap();
    assert_eq!((clip.p, clip.reject), (None, false));

    let d2 = decide_round(&dir, 2).unwrap().record;
    assert_eq!(d2.move_to, None);
    let verdict = d2.verdict.unwrap();
    assert_eq!(verdict.kind, VerdictKind::Plateau);
    assert_eq!(verdict.incumbent, "light/glider/no");
    assert_eq!(
        verdict.not_shown_worse,
        vec!["light/glider/yes".to_string()]
    );
    for c in d2
        .comparisons
        .iter()
        .filter(|c| c.test == Test::NotBetterByDelta)
    {
        assert_eq!((c.p, c.reject), (Some(0.05), true), "{}", c.neighbor);
    }

    // The appended log reads back as the same two records.
    let log = load_decisions(&dir).unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(
        log[1].verdict.as_ref().map(|v| v.kind),
        Some(VerdictKind::Plateau)
    );
}

#[test]
fn decisions_are_append_only() {
    let dir = copy_fixture("append_only");
    decide_round(&dir, 1).unwrap();
    let before = fs::read_to_string(dir.join("DECISIONS.md")).unwrap();
    assert!(matches!(
        decide_round(&dir, 1),
        Err(ExperimentError::AlreadyDecided(1))
    ));
    decide_round(&dir, 2).unwrap();
    let after = fs::read_to_string(dir.join("DECISIONS.md")).unwrap();
    assert!(after.starts_with(&before));
}

#[test]
fn power_from_noise_gives_three_throws_per_arm() {
    let rep = power_for_dir(Path::new(FIXTURE)).unwrap();
    assert_eq!(rep.replicates, 3);
}

#[test]
fn round_after_a_move_centers_on_the_new_incumbent() {
    let dir = copy_fixture("round_after_move");
    fs::remove_file(dir.join("ROUND_2.md")).unwrap();
    decide_round(&dir, 1).unwrap();
    let req = RoundRequest {
        seed: Some(20261013),
        blocks: Some(vec!["sunday".into()]),
        ..RoundRequest::default()
    };
    let (path, new) = write_next_round(&dir, &req).unwrap();
    assert_eq!(path, dir.join("ROUND_2.md"));
    assert_eq!(new.header.purpose, Purpose::ReasonableNeighbors);
    // Five arms of three throws from NOISE.md, all on sunday.
    assert_eq!(new.runs.len(), 15);
    let back = check(&dir).unwrap();
    assert_eq!(back.rounds[1].pending, 15);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("incumbent: light/glider/no"), "{text}");
    // Writing again never overwrites: the next call writes nothing because
    // round 2 is not decided.
    assert!(write_next_round(&dir, &req).is_err());
}

#[test]
fn plateau_ends_the_start() {
    let dir = copy_fixture("plateau_ends");
    decide_round(&dir, 1).unwrap();
    decide_round(&dir, 2).unwrap();
    let err = write_next_round(&dir, &RoundRequest::default()).unwrap_err();
    assert!(
        matches!(
            err,
            ExperimentError::Sheet(SheetError::StartEnded {
                start: 1,
                verdict: VerdictKind::Plateau
            })
        ),
        "{err:?}"
    );
}

#[test]
fn second_start_begins_farthest_from_the_baseline() {
    let dir = copy_fixture("second_start");
    decide_round(&dir, 1).unwrap();
    decide_round(&dir, 2).unwrap();
    let req = RoundRequest {
        start: Some(2),
        seed: Some(5),
        blocks: Some(vec!["saturday".into()]),
        ..RoundRequest::default()
    };
    let (_, new) = write_next_round(&dir, &req).unwrap();
    assert_eq!(new.header.start, 2);
    assert_eq!(new.header.round, 3);
    // Every factor differs from light/dart/no: distance 3, the largest.
    let search = Search::load(&dir).unwrap();
    let p = &search.protocol;
    assert_eq!(distance(p, &p.baseline, &new.header.incumbent), 3.0);
    // A third start may not skip ahead of the second.
    let skip = RoundRequest {
        start: Some(4),
        ..req
    };
    assert!(matches!(
        write_next_round(&dir, &skip),
        Err(ExperimentError::Sheet(SheetError::StartNotNext {
            start: 4,
            expected: 3
        }))
    ));
}

#[test]
fn round_two_needs_round_one_decided() {
    let dir = copy_fixture("out_of_order");
    assert!(matches!(
        decide_round(&dir, 2),
        Err(ExperimentError::MissingDecision(1))
    ));
}
