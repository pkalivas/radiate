#[derive(Debug)]
pub enum SelectorError {
    InvalidObjective,
    InvalidSelection,
    InvalidSelectionSize,
}

impl std::fmt::Display for SelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorError::InvalidObjective => write!(f, "Invalid objective"),
            SelectorError::InvalidSelection => write!(f, "Invalid selection"),
            SelectorError::InvalidSelectionSize => write!(f, "Invalid selection size"),
        }
    }
}

impl std::error::Error for SelectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
