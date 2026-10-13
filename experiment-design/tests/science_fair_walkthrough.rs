//! End to end on the science fair walkthrough in the skill's
//! `references/examples.md`: the tool must reach the same decisions a person
//! reaches by hand. Fixture data are synthetic.

use std::fs;
use std::path::{Path, PathBuf};

use experiment_design::{
    check, decide_round, load_decisions, ExperimentError, Rule, Test, VerdictKind,
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
fn round_two_needs_round_one_decided() {
    let dir = copy_fixture("out_of_order");
    assert!(matches!(
        decide_round(&dir, 2),
        Err(ExperimentError::MissingDecision(1))
    ));
}
