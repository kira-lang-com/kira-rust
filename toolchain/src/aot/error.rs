use std::fmt;

#[derive(Debug)]
pub struct AotError(pub String);

impl fmt::Display for AotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AotError {}

impl From<String> for AotError {
    fn from(s: String) -> Self {
        AotError(s)
    }
}

impl From<&str> for AotError {
    fn from(s: &str) -> Self {
        AotError(s.to_string())
    }
}
