use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PatError {
    IoError(std::io::Error),
    UnkownError
} 

impl fmt::Display for PatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PatError::IoError(ref e) => e.fmt(f),
            PatError::UnkownError => write!(f, "UNKNOWN PAT ERROR"),
        }
    }
}

impl Error for PatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
       match *self {
            PatError::IoError(ref error) => Some(error),
            PatError::UnkownError => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
impl From<std::io::Error> for PatError {
    fn from(value: std::io::Error) -> Self {
        PatError::IoError(value)
    }
}
