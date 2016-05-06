//! Batch Renaming tool.
//!
//! Copyright @ 2016 Joey Ezechiels
extern crate regex;
extern crate term;

use regex::Regex;
use std::path::Path; // FIXME: This is an evil HAX since at this level the
                     //        code shouldn't have to know about Paths.

pub mod bare;

const DEFAULT_ANSWER: &'static str = "";

fn main() {
    let mut log = bare::log::RainbowLog::new();
    let args = bare::cli::Args::parse();

    // FIXME: The mismatch between args.file_paths and
    //          the 0th arg to bare::propose_renames.
    let paths: Vec<&Path> = args.file_paths.iter()
        .map(|p| p.as_path())
        .collect();

    let (proposal, files_not_found) =
        bare::propose_renames(&paths, &args.patterns);

    for file in files_not_found.iter() {
        log.warn(&format!("Not found, skipping {:?}\n", file));
    }
    for (parent, renames) in proposal.iter() {
        log.info(&format!("{:?}:\n", parent));
        for &(ref src, ref dst) in renames.iter() {
            if src != dst {
                log.info(&format!("    {:?}    =>    {:?}\n", src, dst));
            } else {
                log.warn(&format!("    No matches for {:?}\n", src));
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
                        log.error(&format!("Couldn't rename {:?}: {:?}\n",
                                           src, e));
                    }
                }
            }
            log.info("Done.\n");
        },
        "n"|"no"|DEFAULT_ANSWER => log.info("Aborted.\n"),
        ans => log.warn(&format!("Don't know what to do with '{:?}'", ans)),
    }
    bare::exit::quit();
}

//  LocalWords:  filename PathBuf ExitCode
