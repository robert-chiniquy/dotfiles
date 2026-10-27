mod site;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: site-build <src-dir> <out-dir>

Reads <src-dir>/index.md (site frontmatter, then one frontmatter block per
card) and every other .md under <src-dir> (one page each), and writes
<out-dir>/index.html and <out-dir>/<path>/index.html. Paths whose name starts
with '_' or '.' are not pages; <src-dir>/_partials/ holds head.html and
deck-help.md.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let [src, out] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    match site::build(&PathBuf::from(src), &PathBuf::from(out)) {
        Ok(written) => {
            for path in written {
                println!("{}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("site-build: {e}");
            ExitCode::FAILURE
        }
    }
}
