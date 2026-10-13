//! `experiment-design` command line: `check` validates an experiment
//! directory, `power` sizes replicates from phase 1, `round` writes the next
//! run sheet, and `decide` reads a filled round and appends its decision.

use std::io::{self, Write as _};
use std::path::PathBuf;

use anyhow::Context as _;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "experiment-design",
    about = "Protocol checks, run sheets, and round decisions for an experiment directory"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate PROTOCOL.md and parse every ROUND_<n>.md present.
    Check { dir: PathBuf },
    /// Replicates per arm for the not-better-by-delta test, from NOISE.md.
    Power { dir: PathBuf },
    /// Write the next ROUND_<n>.md.
    Round {
        dir: PathBuf,
        /// Start to continue or begin (default: the latest round's start).
        #[arg(long)]
        start: Option<u32>,
        /// Replicates per arm (default: the recommendation from NOISE.md).
        #[arg(long)]
        replicates: Option<u32>,
        /// Seed for run order and frontier sample (default: from the clock).
        #[arg(long)]
        seed: Option<u64>,
        /// Blocks this round uses, comma separated (default: every block).
        #[arg(long, value_delimiter = ',')]
        blocks: Option<Vec<String>>,
    },
    /// Decide round <round> from ROUND_<round>.md and append it to DECISIONS.md.
    Decide { dir: PathBuf, round: u32 },
    /// The starts begun so far and the next ones by greedy farthest-point
    /// selection, with the stop rule.
    Starts {
        dir: PathBuf,
        /// How many new starts to list.
        #[arg(long, default_value_t = 1)]
        count: usize,
    },
    /// Per-start and whole-search verdicts.
    Report { dir: PathBuf },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut out = io::stdout().lock();
    match cli.command {
        Command::Check { dir } => {
            let report = experiment_design::check(&dir)?;
            let p = &report.protocol;
            writeln!(
                out,
                "{}: ok: {} factor(s), delta {}, alpha {}, coverage_q {} (frontier sample n = {}), budget {} runs",
                experiment_design::PROTOCOL_FILE,
                p.factors.len(),
                p.delta,
                p.alpha,
                p.coverage_q,
                p.frontier_sample_size(),
                p.budget.runs
            )?;
            for r in &report.rounds {
                writeln!(
                    out,
                    "ROUND_{}.md: ok: {} run(s), {} not yet run, {} dropped",
                    r.number, r.runs, r.pending, r.dropped
                )?;
            }
        }
        Command::Power { dir } => {
            let report = experiment_design::power_for_dir(&dir)?;
            write!(out, "{report}")?;
        }
        Command::Round {
            dir,
            start,
            replicates,
            seed,
            blocks,
        } => {
            let req = experiment_design::RoundRequest {
                start,
                replicates,
                seed,
                blocks,
            };
            let (path, new) = experiment_design::write_next_round(&dir, &req)?;
            writeln!(
                out,
                "{}: start {}, {} run(s), purpose {:?}, seed {}",
                path.display(),
                new.header.start,
                new.runs.len(),
                new.header.purpose,
                new.header.seed.unwrap_or_default()
            )?;
            for note in &new.notes {
                writeln!(out, "note: {note}")?;
            }
        }
        Command::Decide { dir, round } => {
            let decision = experiment_design::decide_round(&dir, round)?;
            let text = experiment_design::section(&decision).context("serializing the decision")?;
            out.write_all(text.as_bytes())?;
        }
        Command::Starts { dir, count } => {
            let search = experiment_design::Search::load(&dir)?;
            let p = &search.protocol;
            let states = search.starts()?;
            for st in states.values() {
                writeln!(
                    out,
                    "start {}: {} (begun; verdict {})",
                    st.start,
                    p.show(&st.point),
                    st.verdict
                        .map_or_else(|| "none yet".to_string(), |v| v.to_string())
                )?;
            }
            let ordered: Vec<&experiment_design::StartState> = states.values().collect();
            let stop = experiment_design::stop_rule(
                &search,
                &ordered,
                experiment_design::STOP_AFTER_KNOWN_BASINS,
            );
            writeln!(
                out,
                "stop rule: {} ({})",
                if stop.holds { "holds" } else { "does not hold" },
                stop.reason
            )?;
            let next = states.keys().max().map_or(1, |m| m + 1);
            for (i, pick) in experiment_design::next_starts(&search, count)?
                .iter()
                .enumerate()
            {
                writeln!(
                    out,
                    "next start {}: {} (distance {:.3} from the nearest start)",
                    next + i as u32,
                    p.show(&pick.config),
                    pick.distance
                )?;
                for (c, why) in &pick.skipped {
                    writeln!(out, "  skipped {}: {why:?}", p.show(c))?;
                }
            }
        }
        Command::Report { dir } => {
            let (search, report) = experiment_design::report_for_dir(&dir)?;
            write!(
                out,
                "{}",
                experiment_design::ReportDisplay {
                    protocol: &search.protocol,
                    report: &report,
                }
            )?;
        }
    }
    Ok(())
}
