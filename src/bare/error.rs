//!

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum Error {
    /// I/O error: {0}
    IoError(ioe::IoError),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(ioe::IoError::from(error))
    }
}
