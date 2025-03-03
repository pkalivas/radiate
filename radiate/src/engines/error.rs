use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum EngineError {
    SelectorError(SelectorError),
}

impl Error for EngineError {}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::SelectorError(err) => write!(f, "Selector error: {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SelectorError {
    InvalidObjective(String),
    InvalidSelectionSize,
}

impl Error for SelectorError {}

impl Display for SelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorError::InvalidObjective(obj) => write!(f, "Invalid objective: {}", obj),
            SelectorError::InvalidSelectionSize => write!(f, "Invalid selection size"),
        }
    }
}

impl From<SelectorError> for EngineError {
    fn from(err: SelectorError) -> Self {
        EngineError::SelectorError(err)
    }
}
