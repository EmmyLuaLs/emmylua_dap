use std::error::Error;

#[derive(Debug)]
pub enum DebuggerError {
    IoError(std::io::Error),
    AddrParseError(std::net::AddrParseError),
    ConnectionError(String),
}

impl std::fmt::Display for DebuggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebuggerError::IoError(err) => write!(f, "IO Error: {}", err),
            DebuggerError::AddrParseError(err) => write!(f, "Parse Error: {}", err),
            DebuggerError::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
        }
    }
}

impl Error for DebuggerError {}

impl From<std::io::Error> for DebuggerError {
    fn from(err: std::io::Error) -> Self {
        DebuggerError::IoError(err)
    }
}

impl From<std::net::AddrParseError> for DebuggerError {
    fn from(err: std::net::AddrParseError) -> Self {
        DebuggerError::AddrParseError(err)
    }
}

impl From<DebuggerError> for Box<dyn Error + Send> {
    fn from(err: DebuggerError) -> Self {
        Box::new(err)
    }
}
