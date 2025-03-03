use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum EngineError {
    SelectorError(SelectorError),
    BuilderError(BuilderError),
    ProblemError(ProblemError),
}

impl Error for EngineError {}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::SelectorError(err) => write!(f, "Selector error: {}", err),
            EngineError::BuilderError(err) => write!(f, "Builder error: {}", err),
            EngineError::ProblemError(err) => write!(f, "Problem error: {}", err),
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

#[derive(Debug, Clone)]
pub enum BuilderError {
    MissingCodex,
    MissingFitnessFn,
    InvalidFrontSize(String),
}

impl Error for BuilderError {}

impl Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::MissingCodex => write!(f, "Missing codex"),
            BuilderError::MissingFitnessFn => write!(f, "Missing fitness function"),
            BuilderError::InvalidFrontSize(size) => write!(f, "Invalid front size: {}", size),
        }
    }
}

impl From<BuilderError> for EngineError {
    fn from(err: BuilderError) -> Self {
        EngineError::BuilderError(err)
    }
}

#[derive(Debug, Clone)]
pub enum ProblemError {
    EncodingError(String),
    DecodingError(String),
    EvaluationError(String),
}

impl Error for ProblemError {}

impl Display for ProblemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProblemError::EncodingError(err) => write!(f, "Encoding error: {}", err),
            ProblemError::DecodingError(err) => write!(f, "Decoding error: {}", err),
            ProblemError::EvaluationError(err) => write!(f, "Evaluation error: {}", err),
        }
    }
}

impl From<ProblemError> for EngineError {
    fn from(err: ProblemError) -> Self {
        EngineError::BuilderError(BuilderError::InvalidFrontSize(err.to_string()))
    }
}
