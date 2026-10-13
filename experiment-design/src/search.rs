//! The whole search as the files record it: every `ROUND_<n>.md` with its
//! header, paired with its section of `DECISIONS.md` when it has one, and
//! the state of each start derived from them (design items 32, 36 to 38).
//!
//! Starts are not kept in a file of their own. A start's starting point is
//! the incumbent in the header of its first round, and its incumbent after
//! that follows the moves in the decision log. Round numbers are global, so
//! rounds of parallel starts interleave.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::decide::{DecisionRecord, VerdictKind};
use crate::experiment::{
    load_decisions, load_protocol, load_round, round_numbers, ExperimentError,
};
use crate::protocol::{Configuration, Protocol};
use crate::round::{Header, Round};

/// One round file and its decision.
#[derive(Debug, Clone)]
pub struct RoundEntry {
    pub number: u32,
    pub header: Header,
    pub round: Round,
    pub decision: Option<DecisionRecord>,
}

/// What the decision log says about one start up to some round.
#[derive(Debug, Clone, PartialEq)]
pub struct StartState {
    pub start: u32,
    /// The incumbent of the start's first round.
    pub point: Configuration,
    /// Its rounds, ascending.
    pub rounds: Vec<u32>,
    /// The incumbent after the start's decided rounds.
    pub incumbent: Configuration,
    /// The latest verdict among its decisions. A failed confirmation records
    /// no verdict, so a start keeps LOCAL_OPTIMUM or PLATEAU through one.
    pub verdict: Option<VerdictKind>,
    /// Rounds of this start with no decision yet.
    pub undecided: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Search {
    pub protocol: Protocol,
    /// Ascending by round number.
    pub rounds: Vec<RoundEntry>,
}

impl Search {
    pub fn load(dir: &Path) -> Result<Search, ExperimentError> {
        let protocol = load_protocol(dir)?;
        let mut decisions: BTreeMap<u32, DecisionRecord> = load_decisions(dir)?
            .into_iter()
            .map(|d| (d.round, d))
            .collect();
        let mut rounds = Vec::new();
        for n in round_numbers(dir)? {
            let round = load_round(dir, &protocol, n)?;
            let header = round
                .header
                .clone()
                .ok_or(ExperimentError::HeaderMissing(n))?;
            if header.round != n {
                return Err(ExperimentError::RoundNumberMismatch {
                    file: n,
                    header: header.round,
                });
            }
            rounds.push(RoundEntry {
                number: n,
                header,
                round,
                decision: decisions.remove(&n),
            });
        }
        Ok(Search { protocol, rounds })
    }

    pub fn entry(&self, n: u32) -> Option<&RoundEntry> {
        self.rounds.iter().find(|e| e.number == n)
    }

    /// Each start's state from the rounds numbered below `before`, keyed by
    /// start number.
    pub fn starts_before(&self, before: u32) -> Result<BTreeMap<u32, StartState>, ExperimentError> {
        let mut states: BTreeMap<u32, StartState> = BTreeMap::new();
        for e in self.rounds.iter().filter(|e| e.number < before) {
            let st = states.entry(e.header.start).or_insert_with(|| StartState {
                start: e.header.start,
                point: e.header.incumbent.clone(),
                rounds: Vec::new(),
                incumbent: e.header.incumbent.clone(),
                verdict: None,
                undecided: Vec::new(),
            });
            st.rounds.push(e.number);
            match &e.decision {
                None => st.undecided.push(e.number),
                Some(d) => {
                    if let Some(m) = &d.move_to {
                        st.incumbent = self.protocol.parse_config(m).map_err(|source| {
                            ExperimentError::DecisionMove {
                                round: e.number,
                                source,
                            }
                        })?;
                    }
                    if let Some(v) = &d.verdict {
                        st.verdict = Some(v.kind);
                    }
                }
            }
        }
        Ok(states)
    }

    /// Every start's state from every round present.
    pub fn starts(&self) -> Result<BTreeMap<u32, StartState>, ExperimentError> {
        self.starts_before(u32::MAX)
    }

    /// Runs in round `n` and every earlier round, across all starts: the
    /// budget is spent by the whole search.
    pub fn runs_through(&self, n: u32) -> u64 {
        self.rounds
            .iter()
            .filter(|e| e.number <= n)
            .map(|e| e.round.runs.len() as u64)
            .sum()
    }

    pub fn runs_spent(&self) -> u64 {
        self.runs_through(u32::MAX)
    }

    /// Blocks named by runs in rounds numbered below `before`.
    pub fn blocks_before(&self, before: u32) -> BTreeSet<String> {
        self.rounds
            .iter()
            .filter(|e| e.number < before)
            .flat_map(|e| e.round.runs.iter().filter_map(|r| r.block.clone()))
            .collect()
    }

    pub fn next_round_number(&self) -> u32 {
        self.rounds.last().map_or(1, |e| e.number + 1)
    }
}
