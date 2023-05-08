//! CLI facilities. Provides an argument parser in the form of [`Args`],
//! as well as some UI utilities.
//!
//! [`Args`]: ./struct.Args.html
use crate::bare::{
    exit::{self, ExitCode},
    log::{RainbowLog, Writer},
    Pattern
};
use regex;
use regex::Regex;
use std::env;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use term::color;

trait RegexUtils {
    fn named(self, name: &str) -> Self;

    fn case_insensitive(self) -> Self;
}

impl RegexUtils for Result<Regex, regex::Error> {
    fn named(self, name: &str) -> Self {
        match self {
            Ok(regex) => Regex::new(&format!("(?P<{}>({}))", name, regex)),
            Err(e) => Err(e),
        }
    }

    fn case_insensitive(self) -> Self {
        match self {
            Ok(regex) => Regex::new(&format!("((?i){})", regex)),
            Err(e) => Err(e),
        }
    }
}




trait ArgsFor {
    fn args_for(&self, aliases: &[&str]) -> Option<Vec<String>>;
}

impl ArgsFor for [String] {
    fn args_for(&self, aliases: &[&str]) -> Option<Vec<String>> {
        let is_next_flag_alias = |arg: &str| arg.starts_with("-");
        for (idx, alias_arg) in self.iter().enumerate() {
            if aliases.contains(&alias_arg.as_str()) {
                for (offset, arg) in self[idx + 1 ..].iter().enumerate() {
                    if is_next_flag_alias(arg) {
                        return Some(self[idx .. idx + 1 + offset].to_owned());
                    }
                }
                return Some(self[idx .. self.len()].to_owned());
            }
        }
        None
    }
}



#[derive(Debug)]
pub struct Args {
    raw:              Vec<String>,
    pub file_paths:   Vec<PathBuf>,
    pub patterns:     Vec<Pattern>,
    pub dry_run:      bool,
}

impl Args {
    fn new() -> Self {
        Args {
            raw: env::args().collect(),
            file_paths: vec![],
            patterns:   vec![],
            dry_run:    false,
        }
    }

    fn parse_help(self, aliases: &[&str]) -> Self {
        if self.raw.args_for(aliases).is_some() {
            HelpWriter::new()
                .text(
"BARE is the ultimate BAtch REnaming tool. It works by matching regexes
against file names, and applying them in the order they were provided.\nSee ")
                .uri("https://doc.rust-lang.org/regex/regex/#syntax")
                .text(" for regex syntax.\n\n")
                .category("Usage:")
                  .argument("  bare",  "[-h | --help]")
                  .argument("      ",  "[-d | --dry-run]")
                  .argument("      ",  "[-f FILE+ | --files FILE+]")
                  .argument("      ",  "[-p [PAT REP]+ | --pattern [PAT REP]+]")
                .text("\n")
                .category("Options:")
                  .option("  -h --help",    "Show this screen")
                  .option("  -v --version", "Print the version number")
                  .option("  -d --dry-run", "Don't actually rename any files")
                  .option("  -f --files",   "Specify the files to rename")
                  .option("  -p --pattern", "Match files ");
            exit::quit();
        }
        self
    }

    fn parse_dry_run(mut self, aliases: &[&str]) -> Self {
        self.dry_run = self.raw.args_for(aliases).is_some();
        self
    }

    fn parse_version(self, aliases: &[&str]) -> Self {
        if self.raw.args_for(aliases).is_some() {
            HelpWriter::new()
                .text("bare ")
                .colored("v", color::BRIGHT_YELLOW)
                .colored(env!("CARGO_PKG_VERSION"), color::BRIGHT_YELLOW)
                .text("\n");
            exit::quit();
        }
        self
    }

    fn parse_files(mut self, aliases: &[&str]) -> Self {
        match self.raw.args_for(aliases) {
            None => exit::abort(ExitCode::MissingRequiredCliArgument(
                format!("{:?}", aliases))),
            Some(args) => {
                if args.len() == 1 && aliases.contains(&args[0].as_str()) {
                    exit::abort(ExitCode::NotEnoughFiles);
                }
                for file in &args[1..] { // Slice off the alias
                    self.file_paths.push(PathBuf::from(file));
                }
            },
        };
        self
    }

    fn validate_patterns(raw_patterns: &[String], aliases: &[&str]) {
        if !aliases.contains(&raw_patterns[0].as_str()) {
            // TODO: Error: wrong format somehow
        }
        let patterns = &raw_patterns[1..];
        let len = patterns.len();
        if len < 2 {
            exit::abort(ExitCode::NotEnoughPatterns(
                format!("{:?}", &patterns)));
        }
        if len % 2 != 0 {
            exit::abort(ExitCode::MalformedPattern(
                format!("{:?}", &patterns)));
        }
    }

    fn parse_patterns(mut self, aliases: &[&str]) -> Self {
        match self.raw.args_for(&aliases) {
            None => exit::abort(ExitCode::MissingRequiredCliArgument(
                format!("{:?}", aliases))),
            Some(patterns) => {
                Self::validate_patterns(&patterns, aliases);
                let patterns = &patterns[1..]; // Slice off the alias proper
                let mut idx = 0;
                while idx < patterns.len() {
                    // Since the regexes are not used concurrently,
                    // the names won't clash with each other.
                    let result = Regex::new(&patterns[idx])
                        .case_insensitive()
                        .named("regex");
                    match result {
                        Ok(regex) => {
                            let replacement = patterns[idx + 1].to_string();
                            self.patterns.push( (regex, replacement) );
                            idx += 2;
                        },
                        Err(e) => {
                            let msg = format!("{}", e);
                            exit::abort(ExitCode::MalformedRegex(msg));
                        },
                    };
                }
            }
        }
        self
    }

    pub fn parse() -> Self {
        Args::new()
            .parse_help(    &["-h", "--help"])
            .parse_dry_run( &["-d", "--dry-run"])
            .parse_version( &["-v", "--version"])
            .parse_files(   &["-f", "--files"])
            .parse_patterns(&["-p", "--pattern"])
    }
}




struct HelpWriter {
    writer: Writer
}

impl HelpWriter {
    pub fn new() -> Self { HelpWriter {  writer: Writer::new()  } }

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

    pub fn colored(mut self, text: &str, color: color::Color) -> Self {
        self.writer.write_color(text, color).unwrap();
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.writer.write(text).unwrap();
        self
    }
}

/// Print a question, then wait for user input.
/// Keep asking the question while the user input fails validation.
/// Return the answer upon successful validation.
pub fn ask_user(question: &str, validator: &Regex) -> String {
    let mut log = RainbowLog::new();
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





#[cfg(test)]
mod tests {
    use crate::bare::cli::ArgsFor;

    fn raw_args() -> Vec<String> {
        to_string_vec(&vec![
            // Do *NOT* alter the args as they are.
            // They are mined 'by position' below.
            "bare",                                 // program name
            "-p", "ein", "zwei", "drei", "vier",    // patterns
            "--files", "foo.bar", "baz.qux",        // files
            "-d",                                   // dry run
            "--help",                               // help
            "--version",                            // version
            // ... append more here
        ])
    }

    fn to_string_vec(v: &[&str]) -> Vec<String> {
        let mut r: Vec<String> = vec![];
        for s in v.iter() {
            r.push(s.to_string());
        }
        r
    }

    fn subvec(v: Vec<String>, start: usize, end: usize) -> Vec<String> {
        let mut r: Vec<String> = vec![];
        for i in start .. end {
            r.push(v[i].clone());
        }
        r
    }

    #[test]
    fn test_args_for_help() {
        let raw = raw_args();
        let hargs = raw.args_for(&["-h", "--help"]).unwrap();
        assert_eq!(hargs.len(), 1);
        assert_eq!(raw[10].to_string(),  hargs[0].to_string());
    }

    #[test]
    fn test_args_for_dry_run() {
        let raw = raw_args();
        let dargs = raw.args_for(&["-d", "--dry-run"]).unwrap();
        assert_eq!(dargs.len(), 1);
        assert_eq!(raw[9].to_string(),  dargs[0].to_string());
    }

    #[test]
    fn test_args_for_version() {
        let raw = raw_args();
        let vargs = raw.args_for(&["-v", "--version"]).unwrap();
        assert_eq!(vargs.len(), 1);
        assert_eq!(raw[11].to_string(),  vargs[0].to_string());
    }

    #[test]
    fn test_args_for_patterns() {
        let (raw, start, end) = (raw_args(), 1, 6);
        let pargs = raw.args_for(&["-p", "--pattern"]).unwrap();
        assert_eq!(subvec(raw, start, end),  pargs);
    }

    #[test]
    fn test_args_for_files() {
        let (raw, start, end) = (raw_args(), 6, 9);
        let fargs = raw.args_for(&["-f", "--files"]).unwrap();
        assert_eq!(subvec(raw, start, end),  fargs);
    }

    #[test]
    fn test_args_for_bogus_flag() {
        let raw = raw_args();
        let no_args = raw.args_for(&["-s", "--some-bogus-flag"]);
        assert_eq!(None,  no_args);
    }
}
