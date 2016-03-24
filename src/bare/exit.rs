//! Exiting the program made trivial.

use std::process;

/// Exit codes for the program.
pub enum ExitCode {
    Ok =                   0,
    NotEnoughPatterns =    1,
    MalformedPattern =     2,
    NotEnoughFiles =       3,
}

/// Abnormally exit the program. The `exit_code` value specifies the reason.
pub fn abort(exit_code : ExitCode) {
    process::exit(exit_code as i32);
}

/// Normally exit the program.
pub fn quit() {
    process::exit(ExitCode::Ok as i32);
}
