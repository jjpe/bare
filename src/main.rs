// use std::env;

const USAGE: &'static str = "
BAtch REnaming tool.

Usage:
  bare-rs [-f <FILES>     | --files=<FILES>]
          [-p <PAT> <REP> | --pattern=<PAT> <REP>]

Options:
  -h --help                           Show this screen
  -f FILES --files=FILES              The files to rename
  -p [PAT REP] --pattern=[PAT REP]    Matches any file name against the PAT
                                        regex and replaces it with REP.
";

mod cli {
    use std::env;

    #[derive(Debug)]
    pub struct Args {
        files:        Vec<String>,
        pattern:      String,
        replacement:  String,
    }

    impl Args {
        fn new() -> Self {
            Args {
                files:        vec![],
                pattern:      String::new(),
                replacement:  String::new(),
            }
        }

        fn parse_files(mut self) -> Self {
            self.files = env::args()
                .skip_while(|arg| {
                    !vec!["-f", "--files"].contains(&arg.as_str())
                })
                .skip(1)
                .take_while(|arg| !arg.starts_with("-"))
                .collect();
            self
        }

        fn parse_pattern(mut self) -> Self {
            let args : Vec<String> = env::args()
                .skip_while(|arg| {
                    !vec!["-p", "--pattern"].contains(&arg.as_str())
                })
                .skip(1)
                .take(2)
                .collect();
            self.pattern =     args[0].clone();
            self.replacement = args[1].clone();
            self
        }

        pub fn parse() -> Args {
            Args::new()
                .parse_files()
                .parse_pattern()
        }
    }
}

fn main() {
    let args = cli::Args::parse();
    println!("!args = {:?}", args);

}
