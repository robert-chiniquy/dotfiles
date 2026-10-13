//! One experiment directory: `PROTOCOL.md`, the phase 1 runs in `NOISE.md`,
//! `ROUND_<n>.md`, and the append-only `DECISIONS.md` (design items 36 to
//! 38).

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use crate::decide::{self, Context, DecideError, Decision, DecisionRecord, VerdictKind};
use crate::markdown;
use crate::protocol::{ConfigError, Configuration, Protocol, ProtocolError};
use crate::round::{Purpose, Round, RoundError};
use crate::search::Search;

pub const PROTOCOL_FILE: &str = "PROTOCOL.md";
pub const DECISIONS_FILE: &str = "DECISIONS.md";
/// Phase 1 runs of the baseline (design phase 1): one table in the round
/// layout, every row the baseline.
pub const NOISE_FILE: &str = "NOISE.md";

/// Display gives the context only; the cause is the error's `source`, so a
/// reporter that walks the chain prints each cause once.
#[derive(Debug, thiserror::Error)]
pub enum ExperimentError {
    #[error("{}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{}", path.display())]
    Protocol {
        path: PathBuf,
        #[source]
        source: ProtocolError,
    },
    #[error("{}", path.display())]
    Round {
        path: PathBuf,
        #[source]
        source: RoundError,
    },
    /// Boxed: the YAML error is large and this enum travels in every Result.
    #[error("{}: decision block {index}", path.display())]
    DecisionYaml {
        path: PathBuf,
        index: usize,
        #[source]
        source: Box<serde_saphyr::Error>,
    },
    #[error("{DECISIONS_FILE}: round {round} move")]
    DecisionMove {
        round: u32,
        #[source]
        source: ConfigError,
    },
    /// DECISIONS.md is append-only; a round is decided once.
    #[error("round {0} is already decided in DECISIONS.md")]
    AlreadyDecided(u32),
    /// The incumbent of a start's round follows from the decisions of its
    /// earlier rounds, so none may be missing.
    #[error("round {0} has no decision in DECISIONS.md; decide a start's rounds in order")]
    MissingDecision(u32),
    /// A start ends at its first verdict; only a confirmation round may
    /// follow LOCAL_OPTIMUM or PLATEAU (phase 5).
    #[error("start {start} ended with verdict {verdict}; round {round} cannot continue it")]
    StartEnded {
        round: u32,
        start: u32,
        verdict: decide::VerdictKind,
    },
    #[error("round numbers start at 1")]
    RoundZero,
    #[error("{NOISE_FILE}")]
    Power(#[source] crate::power::PowerError),
    #[error("writing the next round")]
    Sheet(#[source] crate::sheet::SheetError),
    #[error("choosing starts")]
    Starts(#[source] crate::starts::StartsError),
    #[error("ROUND_{0}.md does not exist")]
    RoundMissing(u32),
    /// Start, incumbent, and steps are read from the header (design item
    /// 37), so a round without one cannot be placed in the search.
    #[error("ROUND_{0}.md has no header block (round, start, incumbent, purpose, steps)")]
    HeaderMissing(u32),
    #[error("ROUND_{file}.md has round {header} in its header")]
    RoundNumberMismatch { file: u32, header: u32 },
    /// The header names an incumbent the decision log does not lead to.
    #[error("ROUND_{round}.md names incumbent {header}, but the decision log leads to {derived}")]
    IncumbentMismatch {
        round: u32,
        header: String,
        derived: String,
    },
    /// Starts are numbered in the order they begin, from 1.
    #[error("ROUND_{round}.md begins start {start}, but the next new start is {expected}")]
    StartNotNext {
        round: u32,
        start: u32,
        expected: u32,
    },
    /// The baseline is always the first start (design item 31).
    #[error(
        "start 1 must begin at the baseline {baseline}, but ROUND_{round}.md begins it at {header}"
    )]
    FirstStartNotBaseline {
        round: u32,
        baseline: String,
        header: String,
    },
    #[error("round {round}")]
    Decide {
        round: u32,
        #[source]
        source: DecideError,
    },
    #[error("{}: serializing the decision", path.display())]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_saphyr::ser::Error,
    },
}

fn read(path: PathBuf) -> Result<String, ExperimentError> {
    fs::read_to_string(&path).map_err(|source| ExperimentError::Io { path, source })
}

pub fn round_path(dir: &Path, n: u32) -> PathBuf {
    dir.join(format!("ROUND_{n}.md"))
}

pub fn load_protocol(dir: &Path) -> Result<Protocol, ExperimentError> {
    let path = dir.join(PROTOCOL_FILE);
    let text = read(path.clone())?;
    Protocol::from_markdown(&text).map_err(|source| ExperimentError::Protocol { path, source })
}

pub fn load_round(dir: &Path, protocol: &Protocol, n: u32) -> Result<Round, ExperimentError> {
    let path = round_path(dir, n);
    let text = read(path.clone())?;
    Round::from_markdown(&text, protocol).map_err(|source| ExperimentError::Round { path, source })
}

/// Round numbers of the `ROUND_<n>.md` files present, ascending.
pub fn round_numbers(dir: &Path) -> Result<Vec<u32>, ExperimentError> {
    let entries = fs::read_dir(dir).map_err(|source| ExperimentError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut numbers = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ExperimentError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let name = entry.file_name();
        let n = name
            .to_str()
            .and_then(|s| s.strip_prefix("ROUND_"))
            .and_then(|s| s.strip_suffix(".md"))
            .and_then(|s| s.parse::<u32>().ok());
        if let Some(n) = n {
            numbers.push(n);
        }
    }
    numbers.sort_unstable();
    Ok(numbers)
}

/// Every decision block in `DECISIONS.md`, in file order; empty when the
/// file does not exist yet.
pub fn load_decisions(dir: &Path) -> Result<Vec<DecisionRecord>, ExperimentError> {
    let path = dir.join(DECISIONS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(ExperimentError::Io { path, source }),
    };
    markdown::yaml_blocks(&text)
        .iter()
        .enumerate()
        .map(|(i, block)| {
            serde_saphyr::from_str(block).map_err(|source| ExperimentError::DecisionYaml {
                path: path.clone(),
                index: i + 1,
                source: Box::new(source),
            })
        })
        .collect()
}

/// Result of `check`: the protocol and, per round file, its run counts.
#[derive(Debug)]
pub struct CheckReport {
    pub protocol: Protocol,
    pub rounds: Vec<RoundSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundSummary {
    pub number: u32,
    pub runs: usize,
    pub pending: usize,
    pub dropped: usize,
}

/// Validate the protocol and parse every round file present (design item 51).
pub fn check(dir: &Path) -> Result<CheckReport, ExperimentError> {
    let protocol = load_protocol(dir)?;
    let rounds = round_numbers(dir)?
        .into_iter()
        .map(|n| {
            let round = load_round(dir, &protocol, n)?;
            Ok(RoundSummary {
                number: n,
                runs: round.runs.len(),
                pending: round.pending().count(),
                dropped: round.runs.iter().filter(|r| r.is_dropped()).count(),
            })
        })
        .collect::<Result<_, ExperimentError>>()?;
    Ok(CheckReport { protocol, rounds })
}

/// Decide round `n` and append the decision to `DECISIONS.md`.
pub fn decide_round(dir: &Path, n: u32) -> Result<Decision, ExperimentError> {
    if n == 0 {
        return Err(ExperimentError::RoundZero);
    }
    let search = Search::load(dir)?;
    let protocol = &search.protocol;
    let entry = search.entry(n).ok_or(ExperimentError::RoundMissing(n))?;
    if entry.decision.is_some() {
        return Err(ExperimentError::AlreadyDecided(n));
    }
    let header = &entry.header;
    let states = search.starts_before(n)?;
    let s = header.start;
    let incumbent = match states.get(&s) {
        Some(st) => {
            if let Some(m) = st.undecided.first() {
                return Err(ExperimentError::MissingDecision(*m));
            }
            // A start ends at its verdict, except that the incumbent of a
            // LOCAL_OPTIMUM or PLATEAU may still be confirmed (phase 5).
            let confirmable = matches!(
                st.verdict,
                Some(VerdictKind::LocalOptimum | VerdictKind::Plateau)
            ) && header.purpose == Purpose::Confirmation;
            if let Some(verdict) = st.verdict.filter(|_| !confirmable) {
                return Err(ExperimentError::StartEnded {
                    round: n,
                    start: s,
                    verdict,
                });
            }
            st.incumbent.clone()
        }
        None => {
            let expected = states.keys().max().map_or(1, |m| m + 1);
            if s != expected {
                return Err(ExperimentError::StartNotNext {
                    round: n,
                    start: s,
                    expected,
                });
            }
            if s == 1 && header.incumbent != protocol.baseline {
                return Err(ExperimentError::FirstStartNotBaseline {
                    round: n,
                    baseline: protocol.show(&protocol.baseline).to_string(),
                    header: protocol.show(&header.incumbent).to_string(),
                });
            }
            header.incumbent.clone()
        }
    };
    if incumbent != header.incumbent {
        return Err(ExperimentError::IncumbentMismatch {
            round: n,
            header: protocol.show(&header.incumbent).to_string(),
            derived: protocol.show(&incumbent).to_string(),
        });
    }

    let earlier_incumbents: Vec<(u32, Configuration)> = states
        .values()
        .filter(|st| st.start < s)
        .map(|st| (st.start, st.incumbent.clone()))
        .collect();
    let used_blocks = search.blocks_before(n);
    let cx = Context {
        protocol,
        incumbent: &incumbent,
        runs_spent: search.runs_through(n),
        earlier_incumbents: &earlier_incumbents,
        used_blocks: &used_blocks,
    };
    let decision = decide::decide(&cx, &entry.round)
        .map_err(|source| ExperimentError::Decide { round: n, source })?;
    append(&dir.join(DECISIONS_FILE), &decision)?;
    Ok(decision)
}

/// Phase 1 runs from `NOISE.md`.
pub fn load_noise(dir: &Path, protocol: &Protocol) -> Result<Round, ExperimentError> {
    let path = dir.join(NOISE_FILE);
    let text = read(path.clone())?;
    Round::from_markdown(&text, protocol).map_err(|source| ExperimentError::Round { path, source })
}

/// The markdown section appended for one decision.
pub fn section(decision: &Decision) -> Result<String, serde_saphyr::ser::Error> {
    let mut yaml = serde_saphyr::to_string(&decision.record)?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    Ok(format!(
        "\n## Round {}\n\n```yaml\n{yaml}```\n\n{}\n",
        decision.record.round, decision.prose
    ))
}

fn append(path: &Path, decision: &Decision) -> Result<(), ExperimentError> {
    let body = section(decision).map_err(|source| ExperimentError::Serialize {
        path: path.to_path_buf(),
        source,
    })?;
    let io_err = |source| ExperimentError::Io {
        path: path.to_path_buf(),
        source,
    };
    let fresh = !path.exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_err)?;
    if fresh {
        file.write_all(b"# Decisions\n").map_err(io_err)?;
    }
    file.write_all(body.as_bytes()).map_err(io_err)
}
