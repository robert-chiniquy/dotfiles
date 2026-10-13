//! One experiment directory: `PROTOCOL.md`, `ROUND_<n>.md`, and the
//! append-only `DECISIONS.md` (design items 36 to 38).

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use crate::decide::{self, Context, DecideError, Decision, DecisionRecord};
use crate::markdown;
use crate::protocol::{ConfigError, Protocol, ProtocolError};
use crate::round::{Round, RoundError};

pub const PROTOCOL_FILE: &str = "PROTOCOL.md";
pub const DECISIONS_FILE: &str = "DECISIONS.md";

/// Multi-start (design items 31 to 35) is not implemented; every round
/// belongs to the single start from the baseline.
pub const START: u32 = 1;

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
    #[error("{}: round {round} move", path.display())]
    DecisionMove {
        path: PathBuf,
        round: u32,
        #[source]
        source: ConfigError,
    },
    /// DECISIONS.md is append-only; a round is decided once.
    #[error("round {0} is already decided in DECISIONS.md")]
    AlreadyDecided(u32),
    /// The incumbent of round n follows from the decisions of rounds 1 to
    /// n - 1, so none may be missing.
    #[error("round {0} has no decision in DECISIONS.md; decide rounds in order")]
    MissingDecision(u32),
    #[error("round {round} ended start {START} with verdict {verdict:?}; later rounds belong to a new start, which this tool does not run")]
    StartEnded {
        round: u32,
        verdict: decide::VerdictKind,
    },
    #[error("round numbers start at 1")]
    RoundZero,
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
    let protocol = load_protocol(dir)?;
    let decisions = load_decisions(dir)?;
    if decisions.iter().any(|d| d.round == n) {
        return Err(ExperimentError::AlreadyDecided(n));
    }

    // The incumbent starts at the baseline and follows each recorded move.
    let decisions_path = dir.join(DECISIONS_FILE);
    let mut incumbent = protocol.baseline.clone();
    for r in 1..n {
        let d = decisions
            .iter()
            .find(|d| d.round == r)
            .ok_or(ExperimentError::MissingDecision(r))?;
        if let Some(v) = &d.verdict {
            return Err(ExperimentError::StartEnded {
                round: r,
                verdict: v.kind,
            });
        }
        if let Some(m) = &d.move_to {
            incumbent =
                protocol
                    .parse_config(m)
                    .map_err(|source| ExperimentError::DecisionMove {
                        path: decisions_path.clone(),
                        round: r,
                        source,
                    })?;
        }
    }

    let mut runs_spent = 0u64;
    let mut current = None;
    for r in 1..=n {
        let round = load_round(dir, &protocol, r)?;
        runs_spent += round.runs.len() as u64;
        if r == n {
            current = Some(round);
        }
    }
    let Some(round) = current else {
        return Err(ExperimentError::RoundZero);
    };

    let cx = Context {
        protocol: &protocol,
        round_number: n,
        start: START,
        incumbent: &incumbent,
        runs_spent,
    };
    let decision = decide::decide(&cx, &round)
        .map_err(|source| ExperimentError::Decide { round: n, source })?;
    append(&decisions_path, &decision)?;
    Ok(decision)
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
