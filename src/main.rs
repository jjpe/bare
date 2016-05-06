//! Batch Renaming tool.
//!
//! Copyright @ 2016 Joey Ezechiels
extern crate regex;
extern crate term;

use regex::Regex;

pub mod bare;

const DEFAULT_ANSWER: &'static str = "";

fn main() {
    let mut log = bare::log::RainbowLog::new();
    macro_rules! error {
        ($fmtstr:expr $(, $x:expr )* ) => { {
            log.error(&format!($fmtstr, $($x),*));
        } };
    }
    macro_rules! warn {
        ($fmtstr:expr $(, $x:expr )* ) => { {
            log.warn(&format!($fmtstr, $($x),*));
        } };
    }
    macro_rules! info {
        ($fmtstr:expr $(, $x:expr )* ) => { {
            log.info(&format!($fmtstr, $($x),*));
        } };
    }
    macro_rules! debug {
        ($fmtstr:expr $(, $x:expr )* ) => { {
            log.debug(&format!($fmtstr, $($x),*));
        } };
    }

    let args = bare::cli::Args::parse();
    let (proposal, files_not_found) =
        bare::propose_renames(&args.file_paths, &args.patterns);

    for file in files_not_found.iter() {
        warn!("Not found, skipping {:?}\n", file);
    }

    for (parent, renames) in proposal.iter() {
        info!("{:?}:\n", parent);
        for &(ref src, ref dst) in renames.iter() {
            if src != dst {
                info!("    {:?}    =>    {:?}\n", src, dst);
            } else {
                warn!("    No matches for {:?}\n", src);
            }
        }
    }

    if args.dry_run {
        return
    }

    let validator = Regex::new(r"^(?i)(y|n|yes|no)?\n$").unwrap();
    let answer = bare::cli::ask_user("Accord the changes? [y/N] ", &validator);
    match answer.to_lowercase().trim() {
        "y"|"yes" => {
            for (parent, renames) in proposal.iter() {
                for &(ref src_name, ref dst_name) in renames.iter() {
                    let src = parent.join(src_name);
                    let dst = parent.join(dst_name);
                    if let Err(e) = std::fs::rename(&src, &dst) {
                        error!("Couldn't rename {:?}: {:?}\n", src, e);
                    }
                }
            }
            info!("Done.\n");
        },
        "n"|"no"|DEFAULT_ANSWER => log.info("Aborted.\n"),
        ans => warn!("Don't know what to do with '{:?}'\n", ans),
    }
    bare::exit::quit();
}

//  LocalWords:  filename PathBuf ExitCode
