use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Field {
    pub name: Arc<[u8]>,
}

impl Field {
    pub fn new(name: String) -> Self {
        Field {
            name: name.into_bytes().into(),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name).unwrap()
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field {{\n name: {},\n }}", self.name())
    }
}
