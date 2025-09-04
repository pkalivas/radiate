use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Field {
    pub name: Arc<String>,
}

impl Field {
    pub fn new(name: String) -> Self {
        Field {
            name: Arc::new(name),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field {{\n name: {},\n }}", self.name())
    }
}
