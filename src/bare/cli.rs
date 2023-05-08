//!

use crate::bare::{
    log::RainbowLog,
    Pattern,
};
use clap::Parser;
use regex::Regex;
use std::io::{self, Write};
use std::path::PathBuf;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");


#[derive(Debug)]
pub(crate) struct TypedCliArgs {
    pub dry_run: bool,
    pub files: Vec<PathBuf>,
    pub patterns: Vec<Pattern>,
    pub verbosity: u8,
    pub lower_case: bool,
    pub upper_case: bool,
}

impl From<CliArgs> for TypedCliArgs {
    fn from(args: CliArgs) -> Self {
        fn get_patterns(patterns: &[String], output: &mut Vec<Pattern>) {
            match patterns {
                [] => {/* Done */}
                [regex] => panic!("No replacement for regex: {regex}"),
                [regex, replacement, rest @ ..] => {
                    output.push(Pattern {
                        regex: Regex::new(&regex)
                            .expect(&format!("Failed to compile regex: {regex}")),
                        replacement: replacement.to_string(),
                    });
                    get_patterns(rest, output);
                }
            }
        }
        Self {
            dry_run: args.dry_run,
            files: args.files,
            patterns: {
                let mut patterns = vec![];
                get_patterns(&args.patterns, &mut patterns);
                patterns
            },
            verbosity: args.verbosity,
            lower_case: args.lower_case,
            upper_case: args.upper_case,
        }
    }
}


#[derive(Debug, Clone, Parser)]
#[command(author, version, about = Self::about())]
pub(crate) struct CliArgs {
    #[arg(short, long)]
    pub dry_run: bool,

    #[arg(required = true, num_args = 1.., short, long)]
    pub files: Vec<PathBuf>,

    #[arg(
        required = true, num_args = 1.., short, long,
        value_name = "[REGEX] [REPLACEMENT]"
    )]
    pub patterns: Vec<String>,

    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[arg(short, long, help = "Rename all files with their lower-case equivalents")]
    pub lower_case: bool,

    #[arg(short, long, help = "Rename all files with their upper-case equivalents")]
    pub upper_case: bool,

}

impl CliArgs {
    fn about() -> String {
        let mut buf = String::new();
        buf.push_str(&format!("{APP_NAME} v{APP_VERSION}\n"));
        buf.push_str(&format!("{APP_NAME} is the ultimate batch renaming tool.\n"));
        buf.push_str(&format!("It works by matching regex/replacement patterns against file names, then applying the matching ones in the order they were provided.\n"));
        buf.push_str("See https://doc.rust-lang.org/regex/regex/#syntax for regex syntax.");
        buf
    }
}

/// Print a question, then wait for user input.
/// Keep asking the question while the user input fails validation.
/// Return the answer upon successful validation.
pub(crate) fn ask_user(question: &str, validator: &Regex) -> String {
    let mut log = RainbowLog::new();
    let mut answer = String::new();
    while !validator.is_match(&answer) {
        log.info(&format!("{}", question));
        io::stdout().flush().unwrap_or_else(|e| {
            log.error(&format!("Error flushing stdout: {:?}", e)).unwrap()
        });
        answer.clear();
        io::stdin()
            .read_line(&mut answer)
            .expect("Failed to read input");
    }
    answer
}
