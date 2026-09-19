use std::fmt::Display;

#[derive(Debug)]
pub enum RotateError {
    ParseArgumentsError(&'static str),
    IO(std::io::Error),
}

impl Display for RotateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ParseArgumentsError(err_msg) => format!("Parsing Arguments: {err_msg}"),
            Self::IO(err) => err.to_string(),
        };
        write!(f, "{message}")
    }
}

impl From<std::io::Error> for RotateError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
