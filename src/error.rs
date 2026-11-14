use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    /// The user pressed Ctrl-C at a question.
    Interrupted,
    Failed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("cancelled"),
            Self::Interrupted => f.write_str("interrupted"),
            Self::Failed(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Failed(message)
    }
}
