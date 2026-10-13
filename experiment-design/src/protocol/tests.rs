#![cfg(test)]

use super::{Objective, Place, Problem, Protocol, ProtocolError, ReasonableMoves, Threshold};

const BASE: &str = r#"
version: 1
objective: {kind: number, direction: higher, unit: m}
threshold: {absolute: 8.0}
delta: 0.5
alpha: 0.05
coverage_q: 0.2
budget: {runs: 60, batch: 15}
blocks: [saturday, sunday]
factors:
  - {name: paper, kind: unordered, values: [light, heavy]}
  - {name: fold, kind: unordered, values: [dart, glider, delta]}
  - {name: nose_clip, kind: unordered, values: ["no", "yes"]}
  - {name: hours, kind: integer, values: [12, 13, 14]}
infeasible:
  - {paper: heavy, fold: delta}
baseline: {paper: light, fold: dart, nose_clip: "no", hours: 12}
reasonable_moves: [fold, hours]
"#;

fn problems(yaml: &str) -> Vec<Problem> {
    match Protocol::from_yaml(yaml) {
        Err(ProtocolError::Invalid(ps)) => ps,
        other => panic!("expected validation problems, got {other:?}"),
    }
}

fn with(from: &str, to: &str) -> String {
    assert!(BASE.contains(from), "test edit target {from:?} not in BASE");
    BASE.replace(from, to)
}

#[test]
fn valid_protocol_parses_with_typed_kinds() {
    let p = Protocol::from_yaml(BASE).unwrap();
    assert!(matches!(p.objective, Objective::Number { .. }));
    assert_eq!(p.threshold, Threshold::Absolute(8.0));
    assert_eq!(p.factors[3].values, vec!["12", "13", "14"]);
    assert!(p.factors[3].kind.is_ordered());
    assert!(matches!(p.reasonable_moves, ReasonableMoves::Factors(ref f) if f.len() == 2));
    let heavy_delta = p.parse_config("heavy/delta/no/12").unwrap();
    assert!(!p.is_feasible(&heavy_delta));
    assert_eq!(p.show(&p.baseline).to_string(), "light/dart/no/12");
}

#[test]
fn alpha_and_coverage_default_when_absent() {
    let p = Protocol::from_yaml(&with("alpha: 0.05\ncoverage_q: 0.2\n", "")).unwrap();
    assert_eq!((p.alpha, p.coverage_q), (0.05, 0.2));
}

#[test]
fn stop_m_defaults_to_three_and_reads_when_given() {
    assert_eq!(Protocol::from_yaml(BASE).unwrap().stop_m, 3);
    let p =
        Protocol::from_yaml(&with("coverage_q: 0.2\n", "coverage_q: 0.2\nstop_m: 5\n")).unwrap();
    assert_eq!(p.stop_m, 5);
}

#[test]
fn rejects_stop_m_of_zero() {
    assert_eq!(
        problems(&with("coverage_q: 0.2\n", "coverage_q: 0.2\nstop_m: 0\n")),
        vec![Problem::StopMZero]
    );
}

#[test]
fn rejects_missing_delta() {
    assert_eq!(
        problems(&with("delta: 0.5\n", "")),
        vec![Problem::MissingDelta]
    );
}

#[test]
fn rejects_missing_required_inputs() {
    let ps = problems(
        &with("threshold: {absolute: 8.0}\n", "").replace("budget: {runs: 60, batch: 15}\n", ""),
    );
    assert!(ps.contains(&Problem::Missing("threshold")), "{ps:?}");
    assert!(ps.contains(&Problem::Missing("budget")), "{ps:?}");
    assert!(problems(&with("version: 1\n", "")).contains(&Problem::Missing("version")));
    assert!(problems(&with("reasonable_moves: [fold, hours]\n", ""))
        .contains(&Problem::Missing("reasonable_moves")));
}

#[test]
fn rejects_number_objective_without_direction() {
    assert_eq!(
        problems(&with("direction: higher, ", "")),
        vec![Problem::DirectionMissing]
    );
}

#[test]
fn rejects_alpha_outside_open_unit_interval() {
    for a in ["0", "1", "1.5", "-0.1"] {
        let ps = problems(&with("alpha: 0.05", &format!("alpha: {a}")));
        assert!(
            matches!(ps.as_slice(), [Problem::AlphaOutOfRange(_)]),
            "alpha {a}: {ps:?}"
        );
    }
}

#[test]
fn rejects_coverage_outside_open_unit_interval() {
    for q in ["0", "1", "2"] {
        let ps = problems(&with("coverage_q: 0.2", &format!("coverage_q: {q}")));
        assert!(
            matches!(ps.as_slice(), [Problem::CoverageOutOfRange(_)]),
            "q {q}: {ps:?}"
        );
    }
}

#[test]
fn rejects_baseline_value_not_in_factor() {
    let ps = problems(&with("fold: dart, nose_clip", "fold: canard, nose_clip"));
    assert_eq!(
        ps,
        vec![Problem::UnknownValue {
            place: Place::Baseline,
            factor: "fold".into(),
            value: "canard".into()
        }]
    );
}

#[test]
fn rejects_baseline_missing_a_factor() {
    let ps = problems(&with(", hours: 12}", "}"));
    assert_eq!(ps, vec![Problem::BaselineFactorMissing("hours".into())]);
}

#[test]
fn rejects_baseline_listed_infeasible() {
    let ps = problems(&with(
        "{paper: heavy, fold: delta}",
        "{paper: light, fold: dart}",
    ));
    assert_eq!(
        ps,
        vec![Problem::BaselineInfeasible("light/dart/no/12".into())]
    );
}

#[test]
fn rejects_unknown_field_as_yaml_error() {
    // A misspelled key must not be silently ignored.
    assert!(matches!(
        Protocol::from_yaml(&with("delta: 0.5", "delta: 0.5\ndelt: 0.4")),
        Err(ProtocolError::Yaml(_))
    ));
}

#[test]
fn rejects_preference_delta_at_or_above_half() {
    let yaml = with(
        "{kind: number, direction: higher, unit: m}",
        "{kind: preference}",
    );
    assert_eq!(
        problems(&yaml),
        vec![Problem::PreferenceDeltaOutOfRange(0.5)]
    );
}

#[test]
fn rejects_value_containing_slash() {
    let ps = problems(&with("[dart, glider, delta]", "[dart, glider, delta/wing]"));
    assert!(
        ps.iter()
            .any(|p| matches!(p, Problem::ValueContainsSlash { .. })),
        "{ps:?}"
    );
}

#[test]
fn requires_exactly_one_yaml_block_in_markdown() {
    let two = format!("```yaml\n{BASE}```\n\n```yaml\na: 1\n```\n");
    assert!(matches!(
        Protocol::from_markdown(&two),
        Err(ProtocolError::YamlBlockCount(2))
    ));
    assert!(matches!(
        Protocol::from_markdown("no block"),
        Err(ProtocolError::YamlBlockCount(0))
    ));
    assert!(Protocol::from_markdown(&format!("# P\n\n```yaml\n{BASE}```\n")).is_ok());
}

const HEADLINES: &str = r#"
version: 1
objective: {kind: number, direction: higher}
threshold: {margin_over_baseline: 0.3}
delta: 0.3
budget: {runs: 1000}
factors:
  - name: headline
    kind: unordered
    attributes:
      - {name: length_words, kind: ordered, values: [short, long], standard: "bands set here"}
      - {name: idiom, kind: unordered, values: [question, imperative, statement], standard: "sentence mood"}
    levels:
      - {id: h01, text: "Ready to save?", length_words: short, idiom: question}
      - {id: h02, text: "Save today", length_words: short, idiom: imperative}
      - {id: h03, text: "Want to save more today?", length_words: long, idiom: question}
      - {id: h04, text: "Start saving more today", length_words: long, idiom: imperative}
      - {id: h05, text: "Savings start here", length_words: short, idiom: statement}
baseline: {headline: h01}
reasonable_moves: all
"#;

#[test]
fn rejects_attribute_value_carried_by_one_level() {
    // statement appears only in h05, so its effect is h05's effect.
    let ps = problems(HEADLINES);
    assert_eq!(
        ps,
        vec![Problem::AttributeValueUnderCarried {
            factor: "headline".into(),
            attribute: "idiom".into(),
            value: "statement".into(),
            levels: 1
        }]
    );
}

#[test]
fn accepts_attributes_each_carried_by_two_levels() {
    let yaml = HEADLINES.replace(
        "      - {id: h05, text: \"Savings start here\", length_words: short, idiom: statement}\n",
        "      - {id: h05, text: \"Savings start here\", length_words: short, idiom: statement}\n      - {id: h06, text: \"Your savings grow over time\", length_words: long, idiom: statement}\n",
    );
    let p = Protocol::from_yaml(&yaml).unwrap();
    assert_eq!(p.factors[0].values.len(), 6);
    assert!(p.factors[0].is_attributed());
}

#[test]
fn rejects_level_attribute_value_not_listed() {
    let ps = problems(&HEADLINES.replace("idiom: imperative}", "idiom: exclamation}"));
    assert!(
        ps.iter().any(|p| matches!(p, Problem::LevelAttributeValueUnknown { value, .. } if value == "exclamation")),
        "{ps:?}"
    );
}
