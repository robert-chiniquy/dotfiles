//! Statistics tool for the experiment-design skill. It reads an experiment
//! directory of markdown files (a protocol, phase 1 noise runs, round sheets
//! with their headers, a decision log), writes the next round sheet, and does
//! the arithmetic that is the same in every domain, so the agent's judgment
//! is spent on intake and design rather than on p-values.
//!
//! Values are read only from fenced YAML blocks and GitHub tables, through a
//! YAML library and a CommonMark parser.

mod decide;
mod experiment;
mod markdown;
mod neighbors;
mod power;
mod protocol;
mod rng;
mod round;
mod search;
mod sheet;
mod stats;

pub use decide::{
    decide, ComparisonRecord, Context, DecideError, Decision, DecisionRecord, Steps, StepsWord,
    VerdictKind, VerdictRecord, COUNTING_MAX_RUNS,
};
pub use experiment::{
    check, decide_round, load_decisions, load_noise, load_protocol, load_round, round_numbers,
    round_path, section, CheckReport, ExperimentError, RoundSummary, DECISIONS_FILE, NOISE_FILE,
    PROTOCOL_FILE,
};
pub use neighbors::{finest_neighbors, neighbors_at, relation, Neighbor, StepMap};
pub use power::{power, power_for_dir, PowerError, PowerReport, POWER};
pub use protocol::{
    Attribute, Budget, ConfigError, Configuration, Direction, Factor, FactorId, FactorKind, Level,
    Objective, ObjectiveKind, Place, Problem, Protocol, ProtocolError, ReasonableMoves, Threshold,
    ValueId, DEFAULT_ALPHA, DEFAULT_COVERAGE_Q,
};
pub use round::{Arm, Header, Outcome, Purpose, Round, RoundError, Run, Side};
pub use search::{RoundEntry, Search, StartState};
pub use sheet::{
    next_round, write_next_round, FrontierSample, NewRound, PlannedRun, RoundRequest, SheetError,
};
pub use stats::{
    binomial, counting, frontier_sample_size, rejects, separation_p, welch, Rule, StatsError, Test,
};
