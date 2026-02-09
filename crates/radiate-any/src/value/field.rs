use crate::DataType;
use radiate_utils::SmallStr;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct Field {
    pub name: SmallStr,
    dtype: DataType,
}

impl Field {
    pub fn new(name: SmallStr, dtype: DataType) -> Self {
        Field { name, dtype }
    }

    #[inline]
    pub fn name(&self) -> &SmallStr {
        &self.name
    }

    #[inline]
    pub fn dtype(&self) -> &DataType {
        &self.dtype
    }
}

impl From<(&str, DataType)> for Field {
    fn from(s: (&str, DataType)) -> Self {
        Self::new(s.0.into(), s.1)
    }
}

impl From<(String, DataType)> for Field {
    fn from(s: (String, DataType)) -> Self {
        Self::new(s.0.into(), s.1)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.dtype)
    }
}
