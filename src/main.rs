// Batch Renaming tool.
//
// Copyright @ 2016 Joey Ezechiels
extern crate regex;

use regex::Regex;

pub type Pattern = (Regex, String);

/// This module deals with exiting the program.
pub mod exit {
    use std::process;

    /// Exit codes for the program.
    pub enum ExitCode {
        /// The program exited normally.
        Ok =               0,
        /// The pattern is malformed.
        MalformedPattern = 2,
        /// Not enough files were specified.
        NotEnoughFiles =   4,
    }

    /// Abnormally exit the program. The `exit_code` value specifies the reason.
    pub fn abort(exit_code : ExitCode) {
        process::exit(exit_code as i32);
    }

    /// Normally exit the program.
    pub fn quit() {
        process::exit(ExitCode::Ok as i32);
    }
}

/// CLI facilities. Provides an argument parser in the form of [`Args`],
/// as well as some UI utilities.
///
/// [`Args`]: ./struct.Args.html
pub mod cli {
    use regex::Regex;
    use std::env;
    use std::io;
    use std::io::{Write};
    use std::path::Path;
    use exit;
    use exit::ExitCode::*;
    use ::Pattern;

    fn name_regex(name: &str, regex: &Regex) -> Regex {
        let raw = format!("(?P<{}>{})", name, &regex);
        Regex::new(&raw).unwrap()
    }

    #[derive(Debug)]
    pub struct Args<'a> {
        pub file_paths:   Vec<&'a Path>,
        pub patterns:     Vec<Pattern>,
        pub print_help:   bool,
        pub dry_run:      bool,
    }

    impl<'a> Args<'a> {
        fn new() -> Self {
            Args {
                file_paths: vec![],
                patterns:   vec![],
                print_help: false,
                dry_run:    false,
            }
        }

        fn parse_help(mut self) -> Self {
            let vec : Vec<String> = env::args()
                .filter(|arg| vec!["-h", "--help"].contains(&arg.as_str()))
                .collect();
            self.print_help = vec.len() > 0;
            self
        }

        fn parse_dry_run(mut self) -> Self {
            let vec : Vec<String> = env::args()
                .filter(|arg| vec!["-d", "--dry-run"].contains(&arg.as_str()))
                .collect();
            self.dry_run = vec.len() > 0;
            self
        }

        fn parse_files(mut self, file_args: Vec<&'a String>) -> Self {
            for file in file_args.iter().cloned() {
                self.file_paths.push(Path::new(file));
            }
            self
        }

        fn parse_patterns(mut self, raw_patterns: Vec<&'a String>) -> Self {
            let num_raw_patterns = raw_patterns.len();
            if num_raw_patterns < 2 || num_raw_patterns % 2 != 0 {
                println!("Error: Malformed pattern detected in {:?}",
                         raw_patterns);
                print_usage();
                exit::abort(MalformedPattern);
            }
            self.patterns = {
                let mut patterns = vec![];
                for idx in 0..num_raw_patterns - 1 {
                    if idx % 2 != 0 {
                        // Odd indices are values, so don't start there.
                        continue;
                    }
                    // Call every regex "regex" for easy reference. Since
                    // they're used successively, the names won't clash.
                    let regex = Regex::new(raw_patterns[idx]).unwrap();
                    let regex = name_regex("regex", &regex);
                    patterns.push( (regex, raw_patterns[idx + 1].clone()) );
                }
                patterns
            };
            self
        }

        pub fn parse(raw_args: &'a Vec<String>) -> Self {
            let args_for = |flag_aliases: Vec<&str>| {
                raw_args.iter()
                    .skip_while(|raw| !flag_aliases.contains(&raw.as_str()))
                    .skip(1)
                    .take_while(|raw| !raw.starts_with("-"))
                    .collect()
            };
            let raw_files:    Vec<&String> = args_for(vec!["-f", "--files"]);
            let raw_patterns: Vec<&String> = args_for(vec!["-p", "--pattern"]);
            Args::new()
                .parse_help()
                .parse_dry_run()
                .parse_files(raw_files)
                .parse_patterns(raw_patterns)
        }
    }

    /// Print the usage string to stdout.
    pub fn print_usage() {
        println!("BARE is the BAtch REnaming tool. It works by matching regexes
against filenames, and applying them in the order they were provided.
For regex syntax, see https://doc.rust-lang.org/regex/regex/index.html#syntax

Usage:
  bare [-f FILE+      | --files=FILE+]
       [-p [PAT REP]+ | --pattern=[PAT REP+]]

Options:
  -h --help      Show this screen
  -f --files     The files to rename
  -p --pattern   Matches files against each PAT regex and replaces each
                   match with the corresponding REP. A minimum of 1 PAT
                   and 1 REP is required.");
    }

    pub fn get_user_input(question: &str, validator: &Regex) -> String {
        let mut answer_buf = String::new();
        while !validator.is_match(&answer_buf) {
            print!("{}", question);
            io::stdout().flush().unwrap_or_else(
                |e| println!("Error flushing stdout: {:?}", e));
            answer_buf.clear();
            io::stdin().read_line(&mut answer_buf)
                .expect("Failed to read input");
        }
        answer_buf
    }
}


/// This module provides the core functionality from the binary in library form.
pub mod bare {
    use regex::Regex;
    use std::path::{Path,PathBuf};
    use ::Pattern;

    pub struct RenameProposal {
        pub renames: Vec<(PathBuf, PathBuf)>,
        pub not_found: Vec<String>
    }

    pub fn propose_renames(paths: &[&Path],
                           patterns: &[Pattern]) -> RenameProposal {
        let mut renames: Vec<(PathBuf, PathBuf)> = vec![];
        let mut not_found: Vec<String> = vec![];
        for src_path in paths.iter() {
            let parent: &Path = src_path.parent().unwrap();
            let src_name: &str =
                src_path.file_name().unwrap().to_str().unwrap();
            if !src_path.exists() {
                not_found.push(String::from(src_path.to_str().unwrap()));
                continue
            }
            let mut dst_name = String::from(src_name);
            for pat in patterns.iter() {
                let (regex, replacement): (&Regex, &str) = (&pat.0, &pat.1);
                if regex.is_match(&dst_name) {
                    dst_name = regex.replace(&dst_name, replacement);
                }
            }
            let dst_path = Path::new(parent.to_str().unwrap())
                .join(Path::new(&dst_name));
            renames.push((src_path.to_path_buf(), dst_path));
        }
        RenameProposal { renames: renames, not_found: not_found }
    }
}


/// Main fn.
fn main() {
    let raw_args = std::env::args().collect();
    let args = cli::Args::parse(&raw_args);

    if args.print_help {
        cli::print_usage();
        exit::quit();
    }

    let proposal = bare::propose_renames(&args.file_paths, &args.patterns);
    for file in proposal.not_found.iter() {
        println!("[WARN] Not found, skipping {:?}", file);
    }
    for (src, dst) in proposal.renames {
        println!("[INFO] {:?}    =>    {:?}", src, dst);
    }

    const DEFAULT: &'static str = "";
    let re = Regex::new(r"^(?i)(y|n|yes|no)?\n$").unwrap();
    let answer = cli::get_user_input("[INFO] Accord the changes? [y/N] ", &re);
    match answer.to_lowercase().trim() {
        "y"|"yes" => {
            println!("Ju Li! Do the thing!");
        },
        "n"|"no"|DEFAULT => println!("[INFO] Aborted renaming files."),
        ans => println!("[WARN] Don't know what to do with answer {:?}", ans),
    }

    exit::quit();
}

//  LocalWords:  filename PathBuf ExitCode
