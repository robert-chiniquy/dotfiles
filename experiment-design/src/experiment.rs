//! One experiment directory: `PROTOCOL.md`, `ROUND_<n>.md`, and the
//! append-only `DECISIONS.md` (design items 36 to 38).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::protocol::{Protocol, ProtocolError};
use crate::round::{Round, RoundError};

pub const PROTOCOL_FILE: &str = "PROTOCOL.md";

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
