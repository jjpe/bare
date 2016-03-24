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

    fn args_for<'b>(raw_args: &'b [String],
                    flag_aliases: Vec<&str>) -> Vec<&'b String> {
        raw_args.iter()
            .skip_while(|raw| !flag_aliases.contains(&raw.as_str()))
            .skip(1)
            .take_while(|raw| !raw.starts_with("-"))
            .collect()
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

        fn parse_help(mut self, raw_args: &'a [String]) -> Self {
            for arg in raw_args {
                if vec!["-h", "--help"].contains(&arg.as_str()) {
                    self.print_help = true;
                    return self
                }
            }
            self.print_help = false;
            self
        }

        fn parse_dry_run(mut self, raw_args: &'a [String]) -> Self {
            for arg in raw_args {
                if vec!["-d", "--dry-run"].contains(&arg.as_str()) {
                    self.dry_run = true;
                    return self
                }
            }
            self.dry_run = false;
            self
        }

        fn parse_files(mut self, raw_args: &'a [String]) -> Self {
            let file_args = args_for(raw_args, vec!["-f", "--files"]);
            for file in file_args.iter().cloned() {
                self.file_paths.push(Path::new(file));
            }
            self
        }

        fn parse_patterns(mut self, raw_args: &'a [String]) -> Self {
            let raw_patterns = args_for(raw_args, vec!["-p", "--pattern"]);
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

        pub fn parse(raw_args: &'a [String]) -> Self {
            Args::new()
                .parse_help(raw_args)
                .parse_dry_run(raw_args)
                .parse_files(raw_args)
                .parse_patterns(raw_args)
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
  -d --dry-run   Don't actually rename any files
  -f --files     The files to rename
  -p --pattern   Matches files against each PAT regex and replaces each
                   match with the corresponding REP. A minimum of 1 PAT
                   and 1 REP is required.");
    }

    pub fn ask_user(question: &str, validator: &Regex) -> String {
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
    let raw_args: Vec<String> = std::env::args().collect();
    let args = cli::Args::parse(&raw_args);

    if args.print_help {
        cli::print_usage();
        exit::quit();
    }

    let proposal = bare::propose_renames(&args.file_paths, &args.patterns);
    for file in proposal.not_found.iter() {
        println!("[WARN] Not found, skipping {:?}", file);
    }
    for (src, dst) in proposal.renames.clone() {
        println!("[INFO] {:?}    =>    {:?}", src, dst);
    }

    if args.dry_run {
        return
    }

    const DEFAULT: &'static str = "";
    let re = Regex::new(r"^(?i)(y|n|yes|no)?\n$").unwrap();
    let answer = cli::ask_user("[INFO] Accord the changes? [y/N] ", &re);
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
