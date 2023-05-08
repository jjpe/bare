//! Batch Renaming tool.
//!
//! Copyright @ 2016-2018 Joey Ezechiels

use crate::bare::{
    cli::{self, CliArgs, TypedCliArgs},
    exit,
    log::RainbowLog,
    propose_renames
};
use clap::Parser;
use regex::Regex;

pub mod bare;

const DEFAULT_ANSWER: &'static str = "";

fn main() {
    let mut log = RainbowLog::new();
    #[allow(unused)]
    macro_rules! error {
        ($fmt:expr $(, $arg:expr)*) => {
            log.error(&format!($fmt, $($arg),*))
        };
    }
    #[allow(unused)]
    macro_rules! warn {
        ($fmt:expr $(, $arg:expr)*) => {
            log.warn(&format!($fmt, $($arg),*))
        };
    }
    #[allow(unused)]
    macro_rules! info {
        ($fmt:expr $(, $arg:expr)*) => {
            log.info(&format!($fmt, $($arg),*))
        };
    }
    #[allow(unused)]
    macro_rules! debug {
        ($fmt:expr $(, $arg:expr)*) => {
            log.debug(&format!($fmt, $($arg),*))
        };
    }

    let args: TypedCliArgs = CliArgs::parse().into();
    let (proposal, not_found) = propose_renames(&args.files, &args.patterns);
    for file in not_found.iter() {
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
        return;
    }
    let validator = Regex::new(r"^(?i)(y|n|yes|no)?\n$").unwrap();
    let answer = cli::ask_user("Accord the changes? [y/N] ", &validator);
    match answer.to_lowercase().trim() {
        "y" | "yes" => {
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
        }
        "n" | "no" | DEFAULT_ANSWER => log.info("Aborted.\n"),
        ans => warn!("Don't know what to do with '{:?}'\n", ans),
    }
    exit::quit();
}
