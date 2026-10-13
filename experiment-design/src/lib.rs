//! Statistics tool for the experiment-design skill. It reads an experiment
//! directory of markdown files (a protocol, filled round sheets, a decision
//! log) and does the arithmetic that is the same in every domain, so the
//! agent's judgment is spent on intake and design rather than on p-values.
//!
//! Values are read only from fenced YAML blocks and GitHub tables, through a
//! YAML library and a CommonMark parser.

mod experiment;
mod markdown;
mod protocol;
mod round;
mod stats;

pub use experiment::{
    check, load_protocol, load_round, round_numbers, round_path, CheckReport, ExperimentError,
    RoundSummary, PROTOCOL_FILE,
};
pub use protocol::{
    Attribute, Budget, ConfigError, Configuration, Direction, Factor, FactorId, FactorKind, Level,
    Objective, ObjectiveKind, Place, Problem, Protocol, ProtocolError, ReasonableMoves, Threshold,
    ValueId, DEFAULT_ALPHA, DEFAULT_COVERAGE_Q,
};
pub use round::{Arm, Outcome, Round, RoundError, Run, Side};
pub use stats::{
    binomial, counting, frontier_sample_size, rejects, separation_p, welch, Rule, StatsError, Test,
};
