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
    /// Its name
    pub name: String,
    /// Its logical [`DataType`]
    pub dtype: DataType,
    /// Additional custom (opaque) metadata.
    pub metadata: Option<Arc<Metadata>>,
}

/// Support for `ArrowSchema::from_iter([field, ..])`
impl From<Field> for (String, Field) {
    fn from(value: Field) -> Self {
        (value.name.clone(), value)
    }
}

impl Field {
    /// Creates a new [`Field`].
    pub fn new(name: String, dtype: DataType) -> Self {
        Field {
            name,
            dtype,
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
            dtype: self.dtype,
            metadata: Some(Arc::new(metadata)),
        }
    }

    /// Returns the [`Field`]'s [`DataType`].
    #[inline]
    pub fn dtype(&self) -> &DataType {
        &self.dtype
    }

    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Field {{\n name: {},\n dtype: {:?},\n metadata: {:?}\n }}",
            self.name, self.dtype, self.metadata,
        )
    }
}
