//! CLI facilities. Provides an argument parser in the form of [`Args`],
//! as well as some UI utilities.
//!
//! [`Args`]: ./struct.Args.html
use bare::log;
use bare::Pattern;
use bare::exit;
use bare::exit::ExitCode::*;
use regex::Regex;
use std::io;
use std::io::{Write};
use std::path::Path;
use term::color;

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
        let mut log = log::RainbowLog::new();
        if num_raw_patterns < 2 {
            log.error(&format!("Not enough patterns specified: {:?}\n",
                               raw_patterns));
            print_usage();
            exit::abort(NotEnoughPatterns);
        }
        if num_raw_patterns % 2 != 0 {
            log.error(&format!("Malformed pattern detected in {:?}\n",
                               raw_patterns));
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




struct HelpWriter {
    writer: log::Writer
}




impl HelpWriter {
    pub fn new() -> Self { HelpWriter {  writer: log::Writer::new()  } }

    pub fn category(mut self, cat: &str) -> Self {
        self.writer.writeln_color(cat, color::YELLOW).unwrap();
        self
    }

    pub fn argument(mut self, prefix: &str, arg: &str) -> Self {
        let p = String::from(prefix) + " ";
        self.writer.write_color(&p, color::GREEN).unwrap();
        self.writer.writeln_color(arg, color::CYAN).unwrap();
        self
    }

    pub fn option(mut self, left: &str, right: &str) -> Self {
        let left = &format!("{:<15} ", left);
        let right = &format!("{} ", right);
        self.writer.write_color(left, color::BRIGHT_WHITE).unwrap();
        self.writer.writeln_color(right, color::WHITE).unwrap();
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.writer.write_color(uri, color::MAGENTA).unwrap();
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.writer.write(text).unwrap();
        self
    }
}

/// Print the usage string to stdout.
pub fn print_usage() {
    HelpWriter::new()
        .text(
"BARE is the ultimate BAtch REnaming tool. It works by matching regexes
against file names, and applying them in the order they were provided. See \n")
        .uri("https://doc.rust-lang.org/regex/regex/#syntax")
        .text(" for regex syntax.\n\n")
        .category("Usage:")
        .argument("  bare",  "[-h | --help]")
        .argument("      ",  "[-d | --dry-run]")
        .argument("      ",  "[-f FILE+ | --files FILE+]")
        .argument("      ",  "[-p [PAT REP]+ | --pattern [PAT REP+]]")
        .text("\n")
        .category("Options:")
        .option("  -h --help",    "Show this screen")
        .option("  -d --dry-run", "Don't actually rename any files")
        .option("  -f --files",   "Specify the files to rename")
        .option("  -p --pattern", "Match files ");
}

/// Print a question, then wait for user input.
/// Keep asking the question while the user input fails validation.
/// Return the answer upon successful validation.
pub fn ask_user(question: &str, validator: &Regex) -> String {
    let mut log = log::RainbowLog::new();
    let mut answer = String::new();
    while !validator.is_match(&answer) {
        log.info(&format!("{}", question));
        io::stdout().flush().unwrap_or_else(
            |e| log.error(&format!("Error flushing stdout: {:?}", e)) );
        answer.clear();
        io::stdin().read_line(&mut answer).expect("Failed to read input");
    }
    answer
}
