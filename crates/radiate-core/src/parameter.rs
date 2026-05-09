use std::collections::BTreeMap;

use radiate_utils::AnyValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct ParameterSet {
    parameters: BTreeMap<String, Parameter>,
}

impl ParameterSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T: ?Sized>(&mut self, parameter: Parameter) {
        self.insert(short_type_name::<T>(), parameter);
    }

    pub fn insert(&mut self, name: impl Into<String>, parameter: impl Into<Parameter>) {
        self.parameters.insert(name.into(), parameter.into());
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        self.parameters.get_mut(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Parameter)> {
        self.parameters.iter()
    }

    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[serde(untagged)]
pub enum Parameter {
    Scalar(AnyValue<'static>),
    Section(BTreeMap<String, Parameter>),
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter::Section(BTreeMap::new())
    }
}

impl Parameter {
    pub fn section() -> Self {
        Self::default()
    }

    pub fn typed<T: ?Sized>() -> Self {
        Self::section().with("type", short_type_name::<T>())
    }

    pub fn scalar(value: impl Into<AnyValue<'static>>) -> Self {
        Parameter::Scalar(value.into())
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<Parameter>) -> Self {
        self.insert(name, value);
        self
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<Parameter>) {
        match self {
            Parameter::Scalar(_) => {
                panic!("cannot insert into scalar parameter");
            }
            Parameter::Section(map) => {
                map.insert(name.into(), value.into());
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        match self {
            Parameter::Scalar(_) => None,
            Parameter::Section(map) => map.get(name),
        }
    }

    pub fn as_scalar(&self) -> Option<&AnyValue<'static>> {
        match self {
            Parameter::Scalar(value) => Some(value),
            Parameter::Section(_) => None,
        }
    }

    pub fn as_section(&self) -> Option<&BTreeMap<String, Parameter>> {
        match self {
            Parameter::Scalar(_) => None,
            Parameter::Section(map) => Some(map),
        }
    }
}

impl From<AnyValue<'static>> for Parameter {
    fn from(value: AnyValue<'static>) -> Self {
        Parameter::Scalar(value)
    }
}

impl From<&'static str> for Parameter {
    fn from(value: &'static str) -> Self {
        Parameter::Scalar(AnyValue::StrOwned(value.to_string()))
    }
}

impl From<String> for Parameter {
    fn from(value: String) -> Self {
        Parameter::Scalar(AnyValue::StrOwned(value))
    }
}

impl From<f32> for Parameter {
    fn from(value: f32) -> Self {
        Parameter::Scalar(value.into())
    }
}

impl From<f64> for Parameter {
    fn from(value: f64) -> Self {
        Parameter::Scalar(value.into())
    }
}

impl From<usize> for Parameter {
    fn from(value: usize) -> Self {
        Parameter::Scalar(value.into())
    }
}

impl From<i32> for Parameter {
    fn from(value: i32) -> Self {
        Parameter::Scalar(value.into())
    }
}

impl From<bool> for Parameter {
    fn from(value: bool) -> Self {
        Parameter::Scalar(value.into())
    }
}

fn short_type_name<T: ?Sized>() -> &'static str {
    std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or(std::any::type_name::<T>())
}
