//! CLI facilities. Provides an argument parser in the form of [`Args`],
//! as well as some UI utilities.
//!
//! [`Args`]: ./struct.Args.html
use bare::log;
use bare::Pattern;
use bare::exit;
use bare::exit::ExitCode;
use regex;
use regex::Regex;
use std::io;
use std::io::{Write};
use std::path::Path;
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




trait ArgsFor<'a> {
    fn args_for<'l>(&'a self, aliases: &'l [&'l str]) -> Option<&'a Self>;
}

impl<'a> ArgsFor<'a> for [&'a str] {
    fn args_for<'l>(&'a self, aliases: &'l [&'l str]) -> Option<&'a Self> {
        for (idx, arg) in self.iter().enumerate() {
            if aliases.contains(arg) {
                let start = idx + 1;
                for (offset, a) in self[start..].iter().enumerate() {
                    if a.starts_with("-") {
                        return Some(&self[start - 1  .. start + offset]);
                    }
                }
                return Some(&self[start - 1 .. self.len()]);
            }
        }
        None
    }
}




#[derive(Debug)]
pub struct Args<'a> {
    pub file_paths:   Vec<&'a Path>,
    pub patterns:     Vec<Pattern>,
    pub dry_run:      bool,
}

impl<'a> Args<'a> {
    fn new() -> Self {
        Args {
            file_paths: vec![],
            patterns:   vec![],
            dry_run:    false,
        }
    }

    fn parse_help(self, raw: &[&str], aliases: &[&str]) -> Self {
        if raw.args_for(aliases).is_some() {
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
                .option("  -v --version", "Print the version number")
                .option("  -d --dry-run", "Don't actually rename any files")
                .option("  -f --files",   "Specify the files to rename")
                .option("  -p --pattern", "Match files ");
            exit::quit();
        }
        self
    }

    fn parse_dry_run(mut self, raw: &[&str], aliases: &[&str]) -> Self {
        self.dry_run = raw.args_for(aliases).is_some();
        self
    }

    fn parse_version(self, raw: &[&str], aliases: &[&str]) -> Self {
        if raw.args_for(aliases).is_some() {
            HelpWriter::new()
                .text("bare ")
                .numeric("v").numeric(env!("CARGO_PKG_VERSION"))
                .text("\n");
            exit::quit();
        }
        self
    }

    fn parse_files(mut self, raw: &'a [&'a str], aliases: &[&str]) -> Self {
        match raw.args_for(aliases) {
            None => exit::abort(ExitCode::MissingRequiredCliArgument(
                format!("{:?}", aliases))),
            Some(args) => {
                if args.len() == 1 && aliases.contains(&args[0]) {
                    exit::abort(ExitCode::NotEnoughFiles);
                }
                for file in &args[1..] { // Slice off the alias
                    self.file_paths.push(Path::new(file));
                }
            },
        };
        self
    }

    fn validate_patterns(raw_patterns: &[&str], aliases: &[&str]) {
        if !aliases.contains(&raw_patterns[0]) {
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

    fn parse_patterns(mut self, raw: &'a [&'a str], aliases: &[&str]) -> Self {
        match raw.args_for(&aliases) {
            None => exit::abort(ExitCode::MissingRequiredCliArgument(
                format!("{:?}", aliases))),
            Some(patterns) => {
                Self::validate_patterns(&patterns, aliases);
                let patterns = &patterns[1..]; // Slice off the alias proper
                let mut idx = 0;
                while idx < patterns.len() {
                    // Since the regexes are not used concurrently,
                    // the names won't clash with each other.
                    let result = Regex::new(patterns[idx])
                        .case_insensitive()
                        .named("regex");
                    match result {
                        Ok(regex) => {
                            let replacement = patterns[idx + 1];
                            let pattern = (regex, replacement.to_string());
                            self.patterns.push(pattern);
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

    pub fn parse(raw_args: &'a [&'a str]) -> Self {
        Args::new()
            .parse_help(raw_args,     &["-h", "--help"])
            .parse_dry_run(raw_args,  &["-d", "--dry-run"])
            .parse_version(raw_args,  &["-v", "--version"])
            .parse_files(raw_args,    &["-f", "--files"])
            .parse_patterns(raw_args, &["-p", "--pattern"])
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

    pub fn numeric(mut self, uri: &str) -> Self {
        self.writer.write_color(uri, color::YELLOW).unwrap();
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





#[cfg(test)]
mod tests {
    use bare::cli::ArgsFor;

    fn basic_setup<'a>() -> Vec<&'a str> {
        vec![
            "bare",                                 // program name
            "-p", "ein", "zwei", "drei", "vier",    // patterns
            "--files", "foo.bar", "baz.qux",        // files
            "-d",                                   // dry run
            "--help",                               // help
            "--version",                            // version
        ]
    }

    #[test]
    fn test_args_for_help() {
        let raw = basic_setup();
        let hargs = raw.args_for(&["-h", "--help"]).unwrap();
        assert_eq!(hargs.len(), 1);
        assert_eq!(raw[10].to_string(),  hargs[0].to_string());
    }

    #[test]
    fn test_args_for_dry_run() {
        let raw = basic_setup();
        let dargs = raw.args_for(&["-d", "--dry-run"]).unwrap();
        assert_eq!(dargs.len(), 1);
        assert_eq!(raw[9].to_string(),  dargs[0].to_string());
    }

    #[test]
    fn test_args_for_version() {
        let raw = basic_setup();
        let vargs = raw.args_for(&["-v", "--version"]).unwrap();
        assert_eq!(vargs.len(), 1);
        assert_eq!(raw[11].to_string(),  vargs[0].to_string());
    }

    #[test]
    fn test_args_for_patterns() {
        let raw = basic_setup();
        let pargs = raw.args_for(&["-p", "--pattern"]).unwrap();
        assert_eq!(&raw[1..6],  pargs);
    }

    #[test]
    fn test_args_for_files() {
        let raw = basic_setup();
        let fargs = raw.args_for(&["-f", "--files"]).unwrap();
        assert_eq!(&raw[6..9],  fargs);
    }

    #[test]
    fn test_args_for_bogus_flag() {
        let raw = basic_setup();
        let no_args = raw.args_for(&["-s", "--some-bogus-flag"]);
        assert_eq!(None,  no_args);
    }
}
