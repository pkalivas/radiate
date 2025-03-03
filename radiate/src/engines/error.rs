use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum EngineError {
    SelectorError(String),
    BuilderError(String),
    ProblemError(String),
    PopulationError(String),
    CombinedError(Vec<EngineError>),
}

impl Error for EngineError {}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::SelectorError(err) => write!(f, "Selector error: {}", err),
            EngineError::BuilderError(err) => write!(f, "Builder error: {}", err),
            EngineError::ProblemError(err) => write!(f, "Problem error: {}", err),
            EngineError::PopulationError(err) => write!(f, "Population error: {}", err),
            EngineError::CombinedError(errs) => {
                write!(f, "Combined errors:")?;
                for err in errs {
                    write!(f, "\n  - {}", err)?;
                }
                Ok(())
            }
        }
    }
}

impl Into<EngineError> for Vec<EngineError> {
    fn into(self) -> EngineError {
        EngineError::CombinedError(self)
    }
}
