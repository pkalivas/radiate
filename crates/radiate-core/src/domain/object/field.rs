use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use super::DataType;

pub type Metadata = BTreeMap<String, String>;

/// Represents Arrow's metadata of a "column".
///
/// A [`Field`] is the closest representation of the traditional "column": a logical type
/// ([`DataType`]) with a name and nullability.
/// A Field has optional [`Metadata`] that can be used to annotate the field with custom metadata.
///
/// Almost all IO in this crate uses [`Field`] to represent logical information about the data
/// to be serialized.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Field {
    pub name: String,
    pub metadata: Option<Arc<Metadata>>,
}

impl Field {
    /// Creates a new [`Field`].
    pub fn new(name: String) -> Self {
        Field {
            name,
            metadata: Default::default(),
        }
    }

    /// Creates a new [`Field`] with metadata.
    #[inline]
    pub fn with_metadata(self, metadata: Metadata) -> Self {
        if metadata.is_empty() {
            return self;
        }
        Self {
            name: self.name,
            metadata: Some(Arc::new(metadata)),
        }
    }

    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl From<Field> for (String, Field) {
    fn from(value: Field) -> Self {
        (value.name.clone(), value)
    }
}

impl From<String> for Field {
    fn from(name: String) -> Self {
        Field::new(name)
    }
}

impl From<&str> for Field {
    fn from(name: &str) -> Self {
        Field::new(name.to_string())
    }
}

impl From<(&str, Metadata)> for Field {
    fn from((name, metadata): (&str, Metadata)) -> Self {
        Field::new(name.to_string()).with_metadata(metadata)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Field {{\n name: {},\n metadata: {:?}\n }}",
            self.name, self.metadata,
        )
    }
}
