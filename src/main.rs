// Batch Renaming tool.
//
// Copyright @ 2016 Joey Ezechiels
extern crate regex;

/// This module deals with exiting the program.
pub mod exit {
    use std::process;

    /// Exit codes for the program.
    pub enum ExitCodes {
        /// The program exited normally.
        Ok = 0,
        /// The pattern is malformed.
        MalformedPattern = 2,
        /// Not enough files were specified.
        NotEnoughFiles = 4,
    }

    /// Abnormally exit the program. The `exit_code` value specifies the reason.
    pub fn abort(exit_code : ExitCodes) {
        exit(exit_code);
    }

    /// Normally exit the program.
    pub fn quit() {
        exit(ExitCodes::Ok);
    }

    fn exit(exit_code: ExitCodes) {
        process::exit(match exit_code {
            ExitCodes::Ok =>                 0,
            ExitCodes::MalformedPattern =>   2,
            ExitCodes::NotEnoughFiles =>     4,
        });
    }
}

/// CLI facilities. Provides an argument parser in the form of [`Args`],
/// as well as some UI utilities.
///
/// [`Args`]: ./struct.Args.html
pub mod cli {
    use regex::Regex;
    use std::env;
    use std::path::Path;
    use exit;
    use exit::ExitCodes::{MalformedPattern /* , NotEnoughFiles */ };

    #[derive(Debug)]
    pub struct Args<'a> {
        pub file_paths:   Vec<&'a Path>,
        pub patterns:     Vec<(Regex, String)>,
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
                    let mut raw = String::from("(?P<regex>");
                    raw = raw + raw_patterns[idx];
                    raw.push_str(")");
                    patterns.push( (Regex::new(&*raw).unwrap(),
                                    raw_patterns[idx + 1].clone()) );
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

    /// Print a 'file not found' warning message for each
    /// file path in `files` that does not exist.
    pub fn print_nonexisting_files(files : &Vec<String>) {
        let nonexisting_files : Vec<&Path> = files.iter()
            .map(|name| Path::new(name.as_str()))
            .filter(|path| !path.exists())
            .collect();
        for nef in nonexisting_files.iter() {
            println!("[WARN] file not found: {:?}", nef);
        }
    }
}


/// This module provides the core functionality from the binary in library form.
pub mod bare {
    use regex::Regex;
    use std::collections::HashMap;
    use std::path::{Path};

    #[derive(Debug)]
    pub struct RenameJob<'a> {
        mappings: HashMap<&'a Path, String>,
    }

    impl<'a> RenameJob<'a> {
        pub fn new() -> Self {
            RenameJob {  mappings: HashMap::new()  }
        }

        /// Specify which file paths to rename.
        pub fn on_files(mut self, paths: Vec<&'a Path>) -> Self {
            // FIXME: ATM, this fn must be called BEFORE using_patterns(),
            //        since otherwise using_patterns() won't see the paths
            //        specified here.
            //        The actual order in which they're specified, or how
            //        many times each fn is called, should not matter.
            for path in paths {
                if !self.mappings.contains_key(path) {
                    self.mappings.insert(path, String::new());
                }
            }
            self
        }

        /// Specify which patterns to use. A pattern is a (Regex, String) tuple.
        /// When the regex matches a file path, the match is substituted for the
        /// replacement String (i.e. the right tuple element).
        pub fn using_patterns(mut self,
                              patterns: Vec<(Regex, String)>) -> Self {
            for (regex, replacement) in patterns {
                self = self.update_pattern(&regex, replacement);
            }
            self
        }

        fn update_pattern(mut self,
                          regex: &Regex,
                          replacement: String) -> Self {
            for (src, dst) in self.mappings.iter_mut() {
                let src_name = src.file_name().unwrap().to_str().unwrap();
                if *dst == "" && regex.is_match(src_name) {
                    *dst = regex.replace(src_name, &*replacement);
                } else if regex.is_match(dst) {
                    *dst = regex.replace(dst, &*replacement);
                }
            }
            self
        }

        /// Dump the state to standard out. Intended as a debugging tool.
        pub fn dump(self) -> Self {
            println!("mappings:");
            for (src, dst) in self.mappings.iter() {
                println!("    {:?}  =>  {:?}", src, dst);
            }
            self
        }

        /// Apply the changes. If this fn completes successfully, any
        /// matching files have been renamed.
        pub fn apply(self) -> Self {
            // TODO: Change the return value to signal success or failure.
            println!("Applying shit etc.");
            // TODO: perform renamings
            self
        }
    }
}


/// Main fn.
fn main() {
    let raw_args = std::env::args().collect();
    let args = cli::Args::parse(&raw_args);
    println!("args = {:?}", args); // TODO:

    if args.print_help {
        cli::print_usage();
        exit::quit();
    }

    bare::RenameJob::new()
        .on_files(args.file_paths)
        .using_patterns(args.patterns)
        .dump()
        .apply();

    exit::quit();
}
