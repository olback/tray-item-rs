use std::error;

#[derive(Debug)]
struct Location {
    file: &'static str,
    line: u32,
}

#[derive(Debug)]
pub struct TIError {
    cause: String,
    location: Option<Location>,
}

impl error::Error for TIError {}

impl TIError {
    #[allow(dead_code)]
    pub(crate) fn new<C: Into<String>>(cause: C) -> Self {
        Self {
            cause: cause.into(),
            location: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_with_location<C: Into<String>>(
        cause: C,
        file: &'static str,
        line: u32,
    ) -> Self {
        Self {
            cause: cause.into(),
            location: Some(Location { file, line }),
        }
    }
}

impl std::fmt::Display for TIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.location {
            Some(ref location) => {
                write!(f, "{} at {}#{}", self.cause, location.file, location.line)
            }
            None => write!(f, "{}", self.cause),
        }
    }
}
