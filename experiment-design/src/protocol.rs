//! The experiment protocol: the one fenced YAML block in `PROTOCOL.md`
//! (design items 1 to 11, 59, 60; layout in the skill's `formats.md`),
//! parsed into typed inputs and validated before any run (design item 51).
//!
//! Invariant: a `Protocol` value has passed every check. Factor values and
//! configurations are interned as indices here, so everything downstream
//! compares ids, never the strings a person typed.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use crate::markdown;

mod tests;

/// What is being optimized (design item 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Objective {
    Number {
        direction: Direction,
        unit: Option<String>,
    },
    PassRate,
    /// A judge picks the better of two; δ is then a win-rate margin
    /// (design item 24).
    Preference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveKind {
    Number,
    PassRate,
    Preference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Higher,
    Lower,
}

/// Factor kind (design item 8). Integer and continuous factors are taken at
/// their finest step (design item 31), so all four kinds carry a finite list
/// of values; every kind except `Unordered` has an order along that list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactorKind {
    Ordered,
    Unordered,
    Integer,
    Continuous,
}

impl FactorKind {
    pub fn is_ordered(self) -> bool {
        !matches!(self, FactorKind::Unordered)
    }
}

/// Acceptance threshold (design item 2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Threshold {
    Absolute(f64),
    MarginOverBaseline(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub runs: u64,
    pub batch: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactorId(pub usize);

/// Index into one factor's (or one attribute's) value list. Meaningful only
/// together with the factor it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueId(pub usize);

/// One value per factor, in protocol factor order. Ordering follows the
/// protocol's value order, which keeps reports stable across runs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Configuration(Vec<ValueId>);

impl Configuration {
    pub fn value(&self, factor: FactorId) -> ValueId {
        self.0[factor.0]
    }

    pub fn with(&self, factor: FactorId, value: ValueId) -> Configuration {
        let mut values = self.0.clone();
        values[factor.0] = value;
        Configuration(values)
    }

    /// Factors on which the two configurations differ.
    pub fn differing<'a>(
        &'a self,
        other: &'a Configuration,
    ) -> impl Iterator<Item = FactorId> + 'a {
        self.0
            .iter()
            .zip(&other.0)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| FactorId(i))
    }
}

/// An attribute describing the levels of an unordered factor (design item
/// 44). Its kind is ordered or unordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub kind: FactorKind,
    pub values: Vec<String>,
    pub standard: Option<String>,
}

/// One level of an attributed factor: its value for every attribute, in
/// attribute order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    pub attributes: Vec<ValueId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Factor {
    pub name: String,
    pub kind: FactorKind,
    /// The factor's values; for an attributed factor, its level ids.
    pub values: Vec<String>,
    /// Empty unless the factor is attributed.
    pub attributes: Vec<Attribute>,
    /// Parallel to `values` when attributed, otherwise empty.
    pub levels: Vec<Level>,
    pub can_produce_levels: bool,
}

impl Factor {
    pub fn is_attributed(&self) -> bool {
        !self.attributes.is_empty()
    }

    /// Whether the search keeps a step size for this factor (design item
    /// 15): ordered factors do; categories and attributed factors do not.
    pub fn has_step(&self) -> bool {
        self.kind.is_ordered() && !self.is_attributed()
    }

    /// Positions in the span: one less than the number of values.
    pub fn span(&self) -> usize {
        self.values.len().saturating_sub(1)
    }

    /// Initial step (design item 15): a quarter of the span, rounded to the
    /// nearest whole number of finest steps, and at least one finest step.
    pub fn initial_step(&self) -> usize {
        ((self.span() + 2) / 4).max(1)
    }

    pub fn value_id(&self, text: &str) -> Option<ValueId> {
        self.values.iter().position(|v| v == text).map(ValueId)
    }
}

/// Which neighbor moves are reasonable (design item 59): tested every round.
/// Moves outside the set form the rest of the frontier (design item 60).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasonableMoves {
    All,
    Factors(Vec<FactorId>),
}

impl ReasonableMoves {
    pub fn includes(&self, factor: FactorId) -> bool {
        match self {
            ReasonableMoves::All => true,
            ReasonableMoves::Factors(fs) => fs.contains(&factor),
        }
    }
}

/// A validated protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct Protocol {
    pub version: u32,
    pub objective: Objective,
    pub threshold: Threshold,
    pub delta: f64,
    pub alpha: f64,
    pub coverage_q: f64,
    pub budget: Budget,
    pub blocks: Vec<String>,
    pub factors: Vec<Factor>,
    /// Each entry is a partial configuration; a configuration matching every
    /// pair of any entry is infeasible (design item 9).
    pub infeasible: Vec<Vec<(FactorId, ValueId)>>,
    pub baseline: Configuration,
    pub reasonable_moves: ReasonableMoves,
    pub guardrails: Vec<String>,
    /// m of the multi-start stop rule (design item 35): stop adding starts
    /// once the last m starts all ended in known basins. At least 1.
    pub stop_m: usize,
}

/// Default error rate for one comparison (design item 7).
pub const DEFAULT_ALPHA: f64 = 0.05;
/// Default frontier coverage (design item 60).
pub const DEFAULT_COVERAGE_Q: f64 = 0.2;
/// Default m of the multi-start stop rule (design item 35).
pub const DEFAULT_STOP_M: usize = 3;

impl Protocol {
    /// Parse `PROTOCOL.md` text: exactly one fenced YAML block.
    pub fn from_markdown(markdown: &str) -> Result<Protocol, ProtocolError> {
        let blocks = markdown::yaml_blocks(markdown);
        match blocks.as_slice() {
            [one] => Protocol::from_yaml(one),
            other => Err(ProtocolError::YamlBlockCount(other.len())),
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Protocol, ProtocolError> {
        let raw: RawProtocol =
            serde_saphyr::from_str(yaml).map_err(|e| ProtocolError::Yaml(Box::new(e)))?;
        validate(raw)
    }

    pub fn factor_ids(&self) -> impl Iterator<Item = FactorId> {
        (0..self.factors.len()).map(FactorId)
    }

    pub fn factor(&self, id: FactorId) -> &Factor {
        &self.factors[id.0]
    }

    pub fn factor_id(&self, name: &str) -> Option<FactorId> {
        self.factors
            .iter()
            .position(|f| f.name == name)
            .map(FactorId)
    }

    /// Initial step of every factor that keeps one (design item 15).
    pub fn initial_steps(&self) -> crate::neighbors::StepMap {
        self.factor_ids()
            .filter(|f| self.factor(*f).has_step())
            .map(|f| (f, self.factor(f).initial_step()))
            .collect()
    }

    /// Oriented so that larger is better: −1 for a lower-is-better number.
    pub fn orientation(&self) -> f64 {
        match self.objective {
            Objective::Number {
                direction: Direction::Lower,
                ..
            } => -1.0,
            _ => 1.0,
        }
    }

    pub fn is_feasible(&self, config: &Configuration) -> bool {
        !self
            .infeasible
            .iter()
            .any(|entry| entry.iter().all(|(f, v)| config.value(*f) == *v))
    }

    /// Parse a configuration id: factor values in protocol order joined by
    /// `/`, as in `light/glider/no`.
    pub fn parse_config(&self, id: &str) -> Result<Configuration, ConfigError> {
        let parts: Vec<&str> = id.split('/').map(str::trim).collect();
        if parts.len() != self.factors.len() {
            return Err(ConfigError::WrongArity {
                id: id.to_string(),
                expected: self.factors.len(),
            });
        }
        parts
            .iter()
            .zip(&self.factors)
            .map(|(part, factor)| {
                factor
                    .value_id(part)
                    .ok_or_else(|| ConfigError::UnknownValue {
                        id: id.to_string(),
                        factor: factor.name.clone(),
                        value: part.to_string(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Configuration)
    }

    /// Build a configuration from per-factor value text.
    pub fn config_from_values<'a>(
        &self,
        values: impl IntoIterator<Item = &'a str>,
    ) -> Result<Configuration, ConfigError> {
        let joined: Vec<&str> = values.into_iter().collect();
        self.parse_config(&joined.join("/"))
    }

    /// Display adapter for a configuration id.
    pub fn show<'a>(&'a self, config: &'a Configuration) -> ConfigDisplay<'a> {
        ConfigDisplay {
            protocol: self,
            config,
        }
    }

    /// Size of the frontier sample (design item 60): the smallest n with
    /// (1 − q)^n ≤ α.
    pub fn frontier_sample_size(&self) -> u32 {
        crate::stats::frontier_sample_size(self.coverage_q, self.alpha)
    }
}

pub struct ConfigDisplay<'a> {
    protocol: &'a Protocol,
    config: &'a Configuration,
}

impl fmt::Display for ConfigDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (factor, value)) in self.protocol.factors.iter().zip(&self.config.0).enumerate() {
            if i > 0 {
                f.write_str("/")?;
            }
            f.write_str(&factor.values[value.0])?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The id does not have one `/`-separated part per factor.
    #[error("configuration {id:?} needs {expected} values separated by '/'")]
    WrongArity { id: String, expected: usize },
    /// A part is not one of its factor's values.
    #[error("configuration {id:?}: {value:?} is not a value of factor {factor}")]
    UnknownValue {
        id: String,
        factor: String,
        value: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// PROTOCOL.md must hold exactly one fenced YAML block; the count found
    /// is carried.
    #[error("PROTOCOL.md must contain exactly one fenced yaml block, found {0}")]
    YamlBlockCount(usize),
    /// The block is not valid YAML, or a field has the wrong type or an
    /// unknown name.
    /// Boxed: the YAML error is large and this enum travels in every Result.
    #[error("protocol yaml")]
    Yaml(#[source] Box<serde_saphyr::Error>),
    /// The YAML parsed but failed one or more checks; every failure is listed.
    #[error("{}", ProblemList(.0))]
    Invalid(Vec<Problem>),
}

struct ProblemList<'a>(&'a [Problem]);

impl fmt::Display for ProblemList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "protocol has {} problem(s):", self.0.len())?;
        for (i, p) in self.0.iter().enumerate() {
            writeln!(f, "  {}. {p}", i + 1)?;
        }
        Ok(())
    }
}

/// Where a factor or value reference appeared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    Baseline,
    Infeasible(usize),
    ReasonableMoves,
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Place::Baseline => f.write_str("baseline"),
            Place::Infeasible(i) => write!(f, "infeasible entry {}", i + 1),
            Place::ReasonableMoves => f.write_str("reasonable_moves"),
        }
    }
}

/// One validation failure.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Problem {
    /// A required input (named) is absent.
    #[error("missing required input `{0}`")]
    Missing(&'static str),
    /// δ is absent. It has no default: every verdict is stated relative to it
    /// (design item 3).
    #[error("missing delta: name the smallest difference that would make you switch")]
    MissingDelta,
    #[error("delta must be a finite number at least 0, got {0}")]
    DeltaInvalid(f64),
    /// For a preference objective δ is a win-rate margin, so 0.5 + δ must
    /// stay below 1.
    #[error("delta for a preference objective is a win-rate margin in [0, 0.5), got {0}")]
    PreferenceDeltaOutOfRange(f64),
    #[error("alpha must be strictly between 0 and 1, got {0}")]
    AlphaOutOfRange(f64),
    #[error("coverage_q must be strictly between 0 and 1, got {0}")]
    CoverageOutOfRange(f64),
    /// With m = 0 the stop rule would hold before any start ran.
    #[error("stop_m must be at least 1")]
    StopMZero,
    #[error("objective kind number needs a direction (higher or lower)")]
    DirectionMissing,
    #[error("direction applies only to objective kind number")]
    DirectionNotApplicable,
    #[error("threshold needs exactly one of absolute or margin_over_baseline")]
    ThresholdNotExactlyOne,
    #[error("budget.runs must be at least 1")]
    BudgetRunsZero,
    #[error("factors must list at least one factor")]
    NoFactors,
    #[error("factor {0} has no name")]
    FactorNameMissing(usize),
    #[error("factor {0} is listed more than once")]
    DuplicateFactor(String),
    #[error("factor {0} has no kind")]
    FactorKindMissing(String),
    #[error("factor {0} needs values, or attributes and levels")]
    FactorValuesMissing(String),
    #[error("factor {0} lists both values and levels")]
    FactorValuesAndLevels(String),
    #[error("attributed factor {0} must be unordered")]
    AttributedFactorOrdered(String),
    #[error("factor {factor} lists value {value:?} more than once")]
    DuplicateValue { factor: String, value: String },
    /// Configuration ids join values with '/', so a value containing one
    /// could not be read back.
    #[error(
        "factor {factor} value {value:?} contains '/', which separates values in configuration ids"
    )]
    ValueContainsSlash { factor: String, value: String },
    #[error("factor {factor}: attribute {index} has no name")]
    AttributeNameMissing { factor: String, index: usize },
    #[error("factor {factor}: attribute {attribute} must be ordered or unordered")]
    AttributeKindInvalid { factor: String, attribute: String },
    #[error("factor {factor}: attribute {attribute} has no values")]
    AttributeValuesMissing { factor: String, attribute: String },
    #[error("factor {factor}: attributes need levels, and levels need attributes")]
    AttributesWithoutLevels { factor: String },
    #[error("factor {factor}: level {index} has no id")]
    LevelIdMissing { factor: String, index: usize },
    #[error("factor {factor}: level {level} has no value for attribute {attribute}")]
    LevelAttributeMissing {
        factor: String,
        level: String,
        attribute: String,
    },
    #[error("factor {factor}: level {level} gives {attribute} the value {value:?}, which the attribute does not list")]
    LevelAttributeValueUnknown {
        factor: String,
        level: String,
        attribute: String,
        value: String,
    },
    #[error("factor {factor}: level {level} has field {field:?}, which is not an attribute")]
    LevelFieldUnknown {
        factor: String,
        level: String,
        field: String,
    },
    /// Design item 48: an attribute value's effect is separable from the
    /// effect of single levels only when at least two levels carry it.
    #[error("factor {factor}: attribute {attribute} value {value:?} is carried by {levels} level(s); its effect cannot be told apart from that of a single level (needs at least 2)")]
    AttributeValueUnderCarried {
        factor: String,
        attribute: String,
        value: String,
        levels: usize,
    },
    #[error("{place}: unknown factor {factor}")]
    UnknownFactor { place: Place, factor: String },
    #[error("{place}: {value:?} is not a value of factor {factor}")]
    UnknownValue {
        place: Place,
        factor: String,
        value: String,
    },
    #[error("baseline gives no value for factor {0}")]
    BaselineFactorMissing(String),
    /// Design item 10: the baseline is the control in every round, so it
    /// must be runnable.
    #[error("baseline {0} matches an infeasible combination")]
    BaselineInfeasible(String),
    #[error("infeasible entry {0} is empty and would exclude every configuration")]
    InfeasibleEmpty(usize),
}

/// Scalars in the YAML may be written bare (`12`, `0.5`, `true`) or quoted;
/// factor values are compared as the text a person wrote.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Scalar {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scalar::Bool(b) => write!(f, "{b}"),
            Scalar::Int(i) => write!(f, "{i}"),
            Scalar::Float(x) => write!(f, "{x}"),
            Scalar::Str(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProtocol {
    version: Option<u32>,
    objective: Option<RawObjective>,
    threshold: Option<RawThreshold>,
    delta: Option<f64>,
    alpha: Option<f64>,
    coverage_q: Option<f64>,
    stop_m: Option<usize>,
    budget: Option<RawBudget>,
    #[serde(default)]
    blocks: Vec<Scalar>,
    factors: Option<Vec<RawFactor>>,
    #[serde(default)]
    infeasible: Vec<BTreeMap<String, Scalar>>,
    baseline: Option<BTreeMap<String, Scalar>>,
    reasonable_moves: Option<RawReasonableMoves>,
    #[serde(default)]
    guardrails: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObjective {
    kind: Option<ObjectiveKind>,
    direction: Option<Direction>,
    unit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawThreshold {
    absolute: Option<f64>,
    margin_over_baseline: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBudget {
    runs: Option<u64>,
    batch: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AllWord {
    All,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawReasonableMoves {
    All(AllWord),
    Factors(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFactor {
    name: Option<String>,
    kind: Option<FactorKind>,
    values: Option<Vec<Scalar>>,
    attributes: Option<Vec<RawAttribute>>,
    levels: Option<Vec<RawLevel>>,
    #[serde(default)]
    can_produce_levels: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAttribute {
    name: Option<String>,
    kind: Option<FactorKind>,
    values: Option<Vec<Scalar>>,
    standard: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawLevel {
    id: Option<Scalar>,
    /// Free text for the level (a headline's wording); not an attribute.
    #[serde(default)]
    #[allow(dead_code)]
    text: Option<String>,
    #[serde(flatten)]
    fields: BTreeMap<String, Scalar>,
}

fn validate(raw: RawProtocol) -> Result<Protocol, ProtocolError> {
    let mut problems = Vec::new();

    let version = raw.version;
    if version.is_none() {
        problems.push(Problem::Missing("version"));
    }

    let objective = match raw.objective {
        None => {
            problems.push(Problem::Missing("objective"));
            None
        }
        Some(o) => match o.kind {
            None => {
                problems.push(Problem::Missing("objective.kind"));
                None
            }
            Some(ObjectiveKind::Number) => match o.direction {
                Some(direction) => Some(Objective::Number {
                    direction,
                    unit: o.unit,
                }),
                None => {
                    problems.push(Problem::DirectionMissing);
                    None
                }
            },
            Some(kind) => {
                if o.direction.is_some() {
                    problems.push(Problem::DirectionNotApplicable);
                }
                Some(match kind {
                    ObjectiveKind::PassRate => Objective::PassRate,
                    _ => Objective::Preference,
                })
            }
        },
    };

    let threshold = match raw.threshold {
        None => {
            problems.push(Problem::Missing("threshold"));
            None
        }
        Some(t) => match (t.absolute, t.margin_over_baseline) {
            (Some(a), None) => Some(Threshold::Absolute(a)),
            (None, Some(m)) => Some(Threshold::MarginOverBaseline(m)),
            _ => {
                problems.push(Problem::ThresholdNotExactlyOne);
                None
            }
        },
    };

    let delta = match raw.delta {
        None => {
            problems.push(Problem::MissingDelta);
            None
        }
        Some(d) if !d.is_finite() || d < 0.0 => {
            problems.push(Problem::DeltaInvalid(d));
            None
        }
        Some(d) => {
            if matches!(objective, Some(Objective::Preference)) && d >= 0.5 {
                problems.push(Problem::PreferenceDeltaOutOfRange(d));
            }
            Some(d)
        }
    };

    let alpha = raw.alpha.unwrap_or(DEFAULT_ALPHA);
    if !(alpha > 0.0 && alpha < 1.0) {
        problems.push(Problem::AlphaOutOfRange(alpha));
    }
    let coverage_q = raw.coverage_q.unwrap_or(DEFAULT_COVERAGE_Q);
    if !(coverage_q > 0.0 && coverage_q < 1.0) {
        problems.push(Problem::CoverageOutOfRange(coverage_q));
    }
    let stop_m = raw.stop_m.unwrap_or(DEFAULT_STOP_M);
    if stop_m == 0 {
        problems.push(Problem::StopMZero);
    }

    let budget = match raw.budget {
        None => {
            problems.push(Problem::Missing("budget"));
            None
        }
        Some(b) => match b.runs {
            None => {
                problems.push(Problem::Missing("budget.runs"));
                None
            }
            Some(0) => {
                problems.push(Problem::BudgetRunsZero);
                None
            }
            Some(runs) => Some(Budget {
                runs,
                batch: b.batch,
            }),
        },
    };

    let factors = match raw.factors {
        None => {
            problems.push(Problem::Missing("factors"));
            Vec::new()
        }
        Some(fs) => {
            if fs.is_empty() {
                problems.push(Problem::NoFactors);
            }
            validate_factors(fs, &mut problems)
        }
    };

    let infeasible: Vec<Vec<(FactorId, ValueId)>> = raw
        .infeasible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            if entry.is_empty() {
                problems.push(Problem::InfeasibleEmpty(i + 1));
            }
            resolve_pairs(&factors, entry, Place::Infeasible(i), &mut problems)
        })
        .collect();

    let baseline = match &raw.baseline {
        None => {
            problems.push(Problem::Missing("baseline"));
            None
        }
        Some(map) => {
            let pairs = resolve_pairs(&factors, map, Place::Baseline, &mut problems);
            let mut values: Vec<Option<ValueId>> = vec![None; factors.len()];
            for (f, v) in pairs {
                values[f.0] = Some(v);
            }
            let missing: Vec<&Factor> = factors
                .iter()
                .zip(&values)
                .filter(|(_, v)| v.is_none())
                .map(|(f, _)| f)
                .collect();
            for f in &missing {
                // An unknown value was already reported; only report factors
                // the baseline does not mention at all.
                if !map.contains_key(&f.name) {
                    problems.push(Problem::BaselineFactorMissing(f.name.clone()));
                }
            }
            if missing.is_empty() {
                Some(Configuration(values.into_iter().flatten().collect()))
            } else {
                None
            }
        }
    };

    let reasonable_moves = match raw.reasonable_moves {
        None => {
            problems.push(Problem::Missing("reasonable_moves"));
            None
        }
        Some(RawReasonableMoves::All(AllWord::All)) => Some(ReasonableMoves::All),
        Some(RawReasonableMoves::Factors(names)) => Some(ReasonableMoves::Factors(
            names
                .iter()
                .filter_map(|n| match factors.iter().position(|f| &f.name == n) {
                    Some(i) => Some(FactorId(i)),
                    None => {
                        problems.push(Problem::UnknownFactor {
                            place: Place::ReasonableMoves,
                            factor: n.clone(),
                        });
                        None
                    }
                })
                .collect(),
        )),
    };

    let blocks: Vec<String> = raw.blocks.iter().map(Scalar::to_string).collect();

    match (
        version,
        objective,
        threshold,
        delta,
        budget,
        baseline,
        reasonable_moves,
    ) {
        (
            Some(version),
            Some(objective),
            Some(threshold),
            Some(delta),
            Some(budget),
            Some(baseline),
            Some(reasonable_moves),
        ) if problems.is_empty() => {
            let protocol = Protocol {
                version,
                objective,
                threshold,
                delta,
                alpha,
                coverage_q,
                budget,
                blocks,
                factors,
                infeasible,
                baseline,
                reasonable_moves,
                guardrails: raw.guardrails,
                stop_m,
            };
            if protocol.is_feasible(&protocol.baseline) {
                Ok(protocol)
            } else {
                Err(ProtocolError::Invalid(vec![Problem::BaselineInfeasible(
                    protocol.show(&protocol.baseline).to_string(),
                )]))
            }
        }
        _ => Err(ProtocolError::Invalid(problems)),
    }
}

/// Resolve `factor: value` pairs to ids, reporting unknown names.
fn resolve_pairs(
    factors: &[Factor],
    map: &BTreeMap<String, Scalar>,
    place: Place,
    problems: &mut Vec<Problem>,
) -> Vec<(FactorId, ValueId)> {
    map.iter()
        .filter_map(|(name, value)| {
            let Some(fi) = factors.iter().position(|f| &f.name == name) else {
                problems.push(Problem::UnknownFactor {
                    place: place.clone(),
                    factor: name.clone(),
                });
                return None;
            };
            let text = value.to_string();
            match factors[fi].value_id(&text) {
                Some(v) => Some((FactorId(fi), v)),
                None => {
                    problems.push(Problem::UnknownValue {
                        place: place.clone(),
                        factor: name.clone(),
                        value: text,
                    });
                    None
                }
            }
        })
        .collect()
}

fn validate_factors(raw: Vec<RawFactor>, problems: &mut Vec<Problem>) -> Vec<Factor> {
    let mut factors: Vec<Factor> = Vec::new();
    for (index, rf) in raw.into_iter().enumerate() {
        let Some(name) = rf.name else {
            problems.push(Problem::FactorNameMissing(index + 1));
            continue;
        };
        if factors.iter().any(|f| f.name == name) {
            problems.push(Problem::DuplicateFactor(name));
            continue;
        }
        let Some(kind) = rf.kind else {
            problems.push(Problem::FactorKindMissing(name));
            continue;
        };
        let factor = match (rf.values, rf.attributes, rf.levels) {
            (Some(_), _, Some(_)) => {
                problems.push(Problem::FactorValuesAndLevels(name));
                continue;
            }
            (Some(values), None, None) => Factor {
                values: check_values(
                    &name,
                    values.iter().map(Scalar::to_string).collect(),
                    problems,
                ),
                name,
                kind,
                attributes: Vec::new(),
                levels: Vec::new(),
                can_produce_levels: rf.can_produce_levels,
            },
            (None, Some(attributes), Some(levels)) => {
                if kind.is_ordered() {
                    problems.push(Problem::AttributedFactorOrdered(name));
                    continue;
                }
                match attributed(&name, attributes, levels, problems) {
                    Some((attributes, ids, levels)) => Factor {
                        values: check_values(&name, ids, problems),
                        name,
                        kind,
                        attributes,
                        levels,
                        can_produce_levels: rf.can_produce_levels,
                    },
                    None => continue,
                }
            }
            (None, None, None) | (Some(_), Some(_), None) => {
                problems.push(Problem::FactorValuesMissing(name));
                continue;
            }
            (None, _, _) => {
                problems.push(Problem::AttributesWithoutLevels { factor: name });
                continue;
            }
        };
        factors.push(factor);
    }
    factors
}

fn check_values(factor: &str, values: Vec<String>, problems: &mut Vec<Problem>) -> Vec<String> {
    if values.is_empty() {
        problems.push(Problem::FactorValuesMissing(factor.to_string()));
    }
    for (i, v) in values.iter().enumerate() {
        if values[..i].contains(v) {
            problems.push(Problem::DuplicateValue {
                factor: factor.to_string(),
                value: v.clone(),
            });
        }
        if v.contains('/') {
            problems.push(Problem::ValueContainsSlash {
                factor: factor.to_string(),
                value: v.clone(),
            });
        }
    }
    values
}

type AttributedParts = (Vec<Attribute>, Vec<String>, Vec<Level>);

fn attributed(
    factor: &str,
    raw_attributes: Vec<RawAttribute>,
    raw_levels: Vec<RawLevel>,
    problems: &mut Vec<Problem>,
) -> Option<AttributedParts> {
    let before = problems.len();
    let mut attributes = Vec::new();
    for (index, ra) in raw_attributes.into_iter().enumerate() {
        let Some(name) = ra.name else {
            problems.push(Problem::AttributeNameMissing {
                factor: factor.to_string(),
                index: index + 1,
            });
            continue;
        };
        let kind = match ra.kind {
            Some(k @ (FactorKind::Ordered | FactorKind::Unordered)) => k,
            _ => {
                problems.push(Problem::AttributeKindInvalid {
                    factor: factor.to_string(),
                    attribute: name,
                });
                continue;
            }
        };
        let values: Vec<String> = ra
            .values
            .unwrap_or_default()
            .iter()
            .map(Scalar::to_string)
            .collect();
        if values.is_empty() {
            problems.push(Problem::AttributeValuesMissing {
                factor: factor.to_string(),
                attribute: name,
            });
            continue;
        }
        attributes.push(Attribute {
            name,
            kind,
            values,
            standard: ra.standard,
        });
    }
    if attributes.is_empty() || raw_levels.is_empty() {
        problems.push(Problem::AttributesWithoutLevels {
            factor: factor.to_string(),
        });
    }

    let mut ids = Vec::new();
    let mut levels = Vec::new();
    for (index, rl) in raw_levels.into_iter().enumerate() {
        let Some(id) = rl.id.map(|s| s.to_string()) else {
            problems.push(Problem::LevelIdMissing {
                factor: factor.to_string(),
                index: index + 1,
            });
            continue;
        };
        for field in rl.fields.keys() {
            if !attributes.iter().any(|a| &a.name == field) {
                problems.push(Problem::LevelFieldUnknown {
                    factor: factor.to_string(),
                    level: id.clone(),
                    field: field.clone(),
                });
            }
        }
        let mut values = Vec::new();
        for attribute in &attributes {
            match rl.fields.get(&attribute.name).map(Scalar::to_string) {
                None => problems.push(Problem::LevelAttributeMissing {
                    factor: factor.to_string(),
                    level: id.clone(),
                    attribute: attribute.name.clone(),
                }),
                Some(text) => match attribute.values.iter().position(|v| *v == text) {
                    Some(v) => values.push(ValueId(v)),
                    None => problems.push(Problem::LevelAttributeValueUnknown {
                        factor: factor.to_string(),
                        level: id.clone(),
                        attribute: attribute.name.clone(),
                        value: text,
                    }),
                },
            }
        }
        ids.push(id);
        levels.push(Level { attributes: values });
    }

    if problems.len() > before {
        return None;
    }
    // An under-carried value leaves the factor well formed, so the factor is
    // kept and references to it elsewhere still resolve.
    for (ai, attribute) in attributes.iter().enumerate() {
        for (vi, value) in attribute.values.iter().enumerate() {
            let carried = levels
                .iter()
                .filter(|l| l.attributes[ai] == ValueId(vi))
                .count();
            if carried < 2 {
                problems.push(Problem::AttributeValueUnderCarried {
                    factor: factor.to_string(),
                    attribute: attribute.name.clone(),
                    value: value.clone(),
                    levels: carried,
                });
            }
        }
    }
    Some((attributes, ids, levels))
}
