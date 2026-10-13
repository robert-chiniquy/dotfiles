//! `experiment-design` command line: `check` validates an experiment
//! directory, `decide` reads a filled round and appends its decision.

use std::io::{self, Write as _};
use std::path::PathBuf;

use anyhow::Context as _;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "experiment-design",
    about = "Protocol checks and round decisions for an experiment directory"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate PROTOCOL.md and parse every ROUND_<n>.md present.
    Check { dir: PathBuf },
    /// Decide round <round> from ROUND_<round>.md and append it to DECISIONS.md.
    Decide { dir: PathBuf, round: u32 },
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
        Command::Decide { dir, round } => {
            let decision = experiment_design::decide_round(&dir, round)?;
            let text = experiment_design::section(&decision).context("serializing the decision")?;
            out.write_all(text.as_bytes())?;
        }
    }
    Ok(())
}
