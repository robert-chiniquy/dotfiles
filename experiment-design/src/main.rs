//! `experiment-design` command line: `check` validates an experiment
//! directory.

use std::io::{self, Write as _};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "experiment-design",
    about = "Protocol checks for an experiment directory"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate PROTOCOL.md and parse every ROUND_<n>.md present.
    Check { dir: PathBuf },
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
    }
    Ok(())
}
