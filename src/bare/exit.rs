//! Exiting the program made trivial.

use crate::bare::error::Result;
use std::io;
use std::io::Write;
use std::process;

/// Exit codes for the program.
#[derive(Debug, Clone)]
pub enum ExitCode {
    Ok,
    MalformedPattern(String),
    MalformedRegex(String),
    MissingRequiredCliArgument(String),
    NotEnoughFiles,
    NotEnoughPatterns(String),
}

fn exit(exit_code: ExitCode) -> Result<()> {
    io::stdout().flush()?;
    process::exit(match exit_code {
        ExitCode::Ok => 0,
        ExitCode::MalformedPattern(ref patterns) => {
            println!("malformed pattern(s): {}", patterns);
            1
        }
        ExitCode::MalformedRegex(ref patterns) => {
            println!("malformed regex: {}", patterns);
            2
        }
        ExitCode::MissingRequiredCliArgument(ref patterns) => {
            println!("Need to provide one of {}", patterns);
            3
        }
        ExitCode::NotEnoughFiles => {
            println!("provide at least 1 file");
            4
        }
        ExitCode::NotEnoughPatterns(ref patterns) => {
            println!("not enough pattern(s) in {}", patterns);
            5
        }
    });
}

/// Abnormally exit the program. The `exit_code` value specifies the reason.
pub fn abort(exit_code: ExitCode) -> Result<()> {
    print!("Aborting, ");
    exit(exit_code)?;
    Ok(())
}

/// Normally exit the program.
pub fn quit() -> Result<()> {
    exit(ExitCode::Ok)?;
    Ok(())
}
