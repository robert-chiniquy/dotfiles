//! The run sheet and results for one round (`ROUND_<n>.md`, design item 37):
//! one GitHub table, one row per run, columns located by header name.
//!
//! A blank result cell means the run has not been made yet. A blank result
//! with a note is a run that was attempted and yielded nothing usable (a
//! spilled tray); the decision log lists it as dropped rather than losing it.
//!
//! The header is one fenced YAML block before the table: the round, the
//! start, the incumbent, the purpose, the current step of every ordered
//! factor, the seed of the run order and frontier sample, and the guardrails
//! (design item 37). The decision reads the steps from it rather than
//! inferring them from the moves present.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::markdown::{self, Table};
use crate::neighbors::StepMap;
use crate::protocol::{ConfigError, Configuration, Objective, Protocol};

mod tests;

/// What a round is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    /// The incumbent and every reasonable neighbor at the current steps
    /// (design item 16).
    ReasonableNeighbors,
    /// At the finest steps after a round with no move: the incumbent, every
    /// reasonable neighbor again, and a seeded sample of n from the rest of
    /// the frontier (design items 59, 60, and the verdicts section).
    FrontierSample,
    /// Phase 5: the incumbent and the baseline in a block no earlier round
    /// used (verdict 26).
    Confirmation,
}

/// The header block as it is written in the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHeader {
    round: u32,
    start: u32,
    incumbent: String,
    purpose: Purpose,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    steps: Option<BTreeMap<String, usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(default)]
    guardrails: Vec<String>,
}

/// A round header resolved against the protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub round: u32,
    pub start: u32,
    pub incumbent: Configuration,
    pub purpose: Purpose,
    /// `None` when the header has no `steps` entry. When present it names
    /// every factor that keeps a step, and no other.
    pub steps: Option<StepMap>,
    pub seed: Option<u64>,
    pub guardrails: Vec<String>,
}

impl Header {
    /// The header as a YAML block body, steps keyed by factor name.
    pub fn to_yaml(&self, protocol: &Protocol) -> Result<String, serde_saphyr::ser::Error> {
        let raw = RawHeader {
            round: self.round,
            start: self.start,
            incumbent: protocol.show(&self.incumbent).to_string(),
            purpose: self.purpose,
            steps: self.steps.as_ref().map(|m| {
                m.iter()
                    .map(|(f, s)| (protocol.factor(*f).name.clone(), *s))
                    .collect()
            }),
            seed: self.seed,
            guardrails: self.guardrails.clone(),
        };
        serde_saphyr::to_string(&raw)
    }

    fn from_yaml(yaml: &str, protocol: &Protocol) -> Result<Header, RoundError> {
        let raw: RawHeader =
            serde_saphyr::from_str(yaml).map_err(|e| RoundError::HeaderYaml(Box::new(e)))?;
        let incumbent = protocol
            .parse_config(&raw.incumbent)
            .map_err(RoundError::HeaderIncumbent)?;
        let steps = match raw.steps {
            None => None,
            Some(named) => {
                let mut steps = StepMap::new();
                for (name, step) in named {
                    let f = protocol
                        .factor_id(&name)
                        .ok_or_else(|| RoundError::StepUnknownFactor(name.clone()))?;
                    if !protocol.factor(f).has_step() {
                        return Err(RoundError::StepNotOrdered(name));
                    }
                    if step == 0 {
                        return Err(RoundError::StepZero(name));
                    }
                    steps.insert(f, step);
                }
                if let Some(f) = protocol
                    .factor_ids()
                    .find(|f| protocol.factor(*f).has_step() && !steps.contains_key(f))
                {
                    return Err(RoundError::StepMissing(protocol.factor(f).name.clone()));
                }
                Some(steps)
            }
        };
        Ok(Header {
            round: raw.round,
            start: raw.start,
            incumbent,
            purpose: raw.purpose,
            steps,
            seed: raw.seed,
            guardrails: raw.guardrails,
        })
    }
}

/// Which side of a duel won.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// What a row runs: one configuration, or a duel between two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arm {
    Single(Configuration),
    Duel {
        left: Configuration,
        right: Configuration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Measured(f64),
    Winner(Side),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub id: String,
    pub block: Option<String>,
    pub arm: Arm,
    /// `None` when the result cell is blank.
    pub outcome: Option<Outcome>,
    pub time: String,
    pub notes: String,
}

impl Run {
    /// Blank result and a note: attempted, no usable result.
    pub fn is_dropped(&self) -> bool {
        self.outcome.is_none() && !self.notes.is_empty()
    }

    /// Blank result and no note: not run yet.
    pub fn is_pending(&self) -> bool {
        self.outcome.is_none() && self.notes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Round {
    /// `None` when the file has no YAML block.
    pub header: Option<Header>,
    pub runs: Vec<Run>,
}

#[derive(Debug, thiserror::Error)]
pub enum RoundError {
    /// A round file holds exactly one table; the count found is carried.
    #[error("round file must contain exactly one table, found {0}")]
    TableCount(usize),
    #[error("round table has no column named {0:?}")]
    MissingColumn(String),
    #[error("round table row {row} has {found} cells, the header has {expected}")]
    RowWidth {
        row: usize,
        found: usize,
        expected: usize,
    },
    #[error("run {run}")]
    Config {
        run: String,
        #[source]
        source: ConfigError,
    },
    #[error("run {run}: result {text:?} is not a number")]
    NotANumber { run: String, text: String },
    #[error("run {run}: winner {text:?} is neither the left nor the right configuration")]
    WinnerNotInDuel { run: String, text: String },
    #[error("run {run}: a duel needs two different configurations")]
    DuelWithItself { run: String },
    #[error("run {run}: block {block:?} is not one of the protocol's blocks")]
    UnknownBlock { run: String, block: String },
    #[error("run id {0:?} appears more than once")]
    DuplicateRun(String),
    #[error("run in table row {0} has no id")]
    RunIdMissing(usize),
    /// Design item 23 (exact test on two proportions) is not implemented.
    #[error("pass_rate objectives are not supported by this tool yet")]
    PassRateUnsupported,
    /// A round file holds at most one YAML block, its header.
    #[error("round file must contain at most one fenced yaml block (the header), found {0}")]
    HeaderBlockCount(usize),
    /// Boxed: the YAML error is large and this enum travels in every Result.
    #[error("round header yaml")]
    HeaderYaml(#[source] Box<serde_saphyr::Error>),
    #[error("round header incumbent")]
    HeaderIncumbent(#[source] ConfigError),
    #[error("round header steps name {0:?}, which is not a factor")]
    StepUnknownFactor(String),
    /// Only ordered factors keep a step; categories and attributed factors
    /// move to any allowed neighbor.
    #[error("round header gives a step for factor {0}, which is not ordered")]
    StepNotOrdered(String),
    #[error("round header gives factor {0} a step of 0")]
    StepZero(String),
    /// A `steps` entry must cover every ordered factor, so that a decision
    /// never has to guess one.
    #[error("round header steps give no step for ordered factor {0}")]
    StepMissing(String),
}

impl Round {
    pub fn from_markdown(markdown: &str, protocol: &Protocol) -> Result<Round, RoundError> {
        let header = match markdown::yaml_blocks(markdown).as_slice() {
            [] => None,
            [one] => Some(Header::from_yaml(one, protocol)?),
            other => return Err(RoundError::HeaderBlockCount(other.len())),
        };
        let tables = markdown::tables(markdown);
        let mut round = match tables.as_slice() {
            [table] => Round::from_table(table, protocol)?,
            other => return Err(RoundError::TableCount(other.len())),
        };
        round.header = header;
        Ok(round)
    }

    pub fn from_table(table: &Table, protocol: &Protocol) -> Result<Round, RoundError> {
        let col = |name: &str| {
            table
                .column(name)
                .ok_or_else(|| RoundError::MissingColumn(name.to_string()))
        };
        let run_col = col("run")?;
        let block_col = table.column("block");
        if block_col.is_none() && !protocol.blocks.is_empty() {
            return Err(RoundError::MissingColumn("block".to_string()));
        }
        let time_col = table.column("time");
        let notes_col = table.column("notes");

        enum Layout {
            Single {
                factors: Vec<usize>,
                result: usize,
            },
            Duel {
                left: usize,
                right: usize,
                winner: usize,
            },
        }
        let layout = match protocol.objective {
            Objective::Number { .. } => Layout::Single {
                factors: protocol
                    .factors
                    .iter()
                    .map(|f| col(&f.name))
                    .collect::<Result<_, _>>()?,
                result: col("result")?,
            },
            Objective::Preference => Layout::Duel {
                left: col("left")?,
                right: col("right")?,
                winner: col("winner")?,
            },
            Objective::PassRate => return Err(RoundError::PassRateUnsupported),
        };

        let mut runs: Vec<Run> = Vec::new();
        for (i, row) in table.rows.iter().enumerate() {
            if row.len() != table.header.len() {
                return Err(RoundError::RowWidth {
                    row: i + 1,
                    found: row.len(),
                    expected: table.header.len(),
                });
            }
            let id = row[run_col].clone();
            if id.is_empty() {
                return Err(RoundError::RunIdMissing(i + 1));
            }
            if runs.iter().any(|r| r.id == id) {
                return Err(RoundError::DuplicateRun(id));
            }
            let config_err = |source| RoundError::Config {
                run: id.clone(),
                source,
            };
            let block = block_col.map(|c| row[c].clone()).filter(|b| !b.is_empty());
            if let Some(b) = &block {
                if !protocol.blocks.is_empty() && !protocol.blocks.contains(b) {
                    return Err(RoundError::UnknownBlock {
                        run: id,
                        block: b.clone(),
                    });
                }
            }
            let (arm, outcome) = match &layout {
                Layout::Single { factors, result } => {
                    let config = protocol
                        .config_from_values(factors.iter().map(|&c| row[c].as_str()))
                        .map_err(config_err)?;
                    let text = &row[*result];
                    let outcome = if text.is_empty() {
                        None
                    } else {
                        let x: f64 = text.parse().map_err(|_| RoundError::NotANumber {
                            run: id.clone(),
                            text: text.clone(),
                        })?;
                        if !x.is_finite() {
                            return Err(RoundError::NotANumber {
                                run: id,
                                text: text.clone(),
                            });
                        }
                        Some(Outcome::Measured(x))
                    };
                    (Arm::Single(config), outcome)
                }
                Layout::Duel {
                    left,
                    right,
                    winner,
                } => {
                    let l = protocol.parse_config(&row[*left]).map_err(config_err)?;
                    let r = protocol.parse_config(&row[*right]).map_err(config_err)?;
                    if l == r {
                        return Err(RoundError::DuelWithItself { run: id });
                    }
                    let text = &row[*winner];
                    let outcome = if text.is_empty() {
                        None
                    } else {
                        let w = protocol.parse_config(text).map_err(|_| {
                            RoundError::WinnerNotInDuel {
                                run: id.clone(),
                                text: text.clone(),
                            }
                        })?;
                        let side = if w == l {
                            Side::Left
                        } else if w == r {
                            Side::Right
                        } else {
                            return Err(RoundError::WinnerNotInDuel {
                                run: id,
                                text: text.clone(),
                            });
                        };
                        Some(Outcome::Winner(side))
                    };
                    (Arm::Duel { left: l, right: r }, outcome)
                }
            };
            runs.push(Run {
                id,
                block,
                arm,
                outcome,
                time: time_col.map(|c| row[c].clone()).unwrap_or_default(),
                notes: notes_col.map(|c| row[c].clone()).unwrap_or_default(),
            });
        }
        Ok(Round { header: None, runs })
    }

    pub fn pending(&self) -> impl Iterator<Item = &Run> {
        self.runs.iter().filter(|r| r.is_pending())
    }
}
