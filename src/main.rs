//! Batch Renaming tool.
//!
//! Copyright @ 2016 Joey Ezechiels
extern crate regex;
extern crate term;

use regex::Regex;

pub mod bare;

fn main() {
    let mut log = bare::log::RainbowLog::new();

    let raw_args: Vec<String> = std::env::args().collect();
    let args = bare::cli::Args::parse(&raw_args);

    if args.print_help {
        bare::cli::print_usage();
        bare::exit::quit();
    }

    let proposal = bare::propose_renames(&args.file_paths, &args.patterns);
    for file in proposal.not_found.iter() {
        log.warn(&format!("Not found, skipping {:?}\n", file));
    }
    for (src, dst) in proposal.renames.clone() {
        log.info(&format!("{:?}    =>    {:?}\n", src, dst));
    }

    if args.dry_run {
        return
    }

    const DEFAULT: &'static str = "";
    let re = Regex::new(r"^(?i)(y|n|yes|no)?\n$").unwrap();
    let answer = bare::cli::ask_user("Accord the changes? [y/N] ", &re);
    match answer.to_lowercase().trim() {
        "y"|"yes" => {
            for (src, dst) in proposal.renames {
                if let Err(e) = std::fs::rename(&src, &dst) {
                    log.error(&format!("Failed to rename {:?}: {:?}\n", src, e));
                }
            }
            log.info("Done renaming files.");
        },
        "n"|"no"|DEFAULT => log.info("Aborted renaming files."),
        ans => log.warn(&format!("Don't know what to do with answer {:?}", ans)),
    }

    bare::exit::quit();
}

//  LocalWords:  filename PathBuf ExitCode
