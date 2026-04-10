use super::DataType;
use radiate_utils::SmallStr;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Field {
    pub name: SmallStr,
    dtype: DataType,
}

impl Field {
    pub const fn new_const(name: &'static str, dtype: DataType) -> Self {
        Field {
            name: SmallStr::from_static(name),
            dtype,
        }
    }

    pub fn new(name: SmallStr, dtype: DataType) -> Self {
        Field { name, dtype }
    }

    pub fn with_dtype(&self, dtype: DataType) -> Self {
        Field {
            name: self.name.clone(),
            dtype,
        }
    }

    pub fn with_name(&self, name: SmallStr) -> Self {
        Field {
            name,
            dtype: self.dtype.clone(),
        }
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
