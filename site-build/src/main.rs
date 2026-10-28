mod links;
mod site;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: site-build <src-dir> <out-dir>

Reads <src-dir>/index.md (site frontmatter, then one frontmatter block per
card) and every other .md under <src-dir> (one page each), and writes
<out-dir>/index.html and <out-dir>/<path>/index.html. Paths whose name starts
with '_' or '.' are not pages; <src-dir>/_partials/ holds head.html and
deck-help.md.

After writing, it warns on stderr about dead links in the written pages: a
site path with no file under <out-dir>, or a remote URL that does not answer
below HTTP 400. Warnings do not change the exit status.";

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
    let out = PathBuf::from(out);
    match site::build(&PathBuf::from(src), &out) {
        Ok(built) => {
            for path in &built.written {
                println!("{}", path.display());
            }
            for warning in links::check(&out, &built.written, built.url.as_deref(), links::http_status) {
                eprintln!("site-build: warning: {warning}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("site-build: {e}");
            ExitCode::FAILURE
        }
    }
}
