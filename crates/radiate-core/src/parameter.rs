use std::sync::Arc;

use radiate_utils::{AnyValue, DataType, Field};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct ParameterSet {
    parameters: Vec<(String, Param)>,
}

impl ParameterSet {
    pub fn new() -> Self {
        ParameterSet {
            parameters: Vec::new(),
        }
    }

    pub fn register<T>(&mut self, param: impl Into<Param>) -> Param
    where
        T: Default + 'static,
    {
        let type_name = std::any::type_name::<T>();
        let parts = type_name.rsplit("::").take(1).collect::<Vec<_>>();

        let name = parts[0];
        let param = param.into();
        self.parameters.push((name.to_string(), param.clone()));
        param
    }

    pub fn add(&mut self, name: &str, param: Param) {
        self.parameters.push((name.to_string(), param));
    }

    pub fn get(&self, name: &str) -> Option<&Param> {
        self.parameters
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p)
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
pub struct Param(Arc<String>, AnyValue<'static>);

impl Param {
    pub fn typed<T: ?Sized>() -> Self {
        let type_name = std::any::type_name::<T>();
        let parts: Vec<&str> = type_name.rsplit("::").take(2).collect();
        let obj_name = radiate_utils::intern!(parts.join("."));

        let named_field = Field::new("type".into(), DataType::String);
        let other = Field::new(obj_name.into(), DataType::String);

        Self::value(
            "param",
            AnyValue::Struct(vec![(named_field, AnyValue::Str(obj_name))]),
        )

        // Param(AnyValue::Struct(
        //     [(
        //         named_field,
        //         AnyValue::Struct(vec![(other, AnyValue::Str(obj_name))]),
        //     )]
        //     .to_vec(),
        // ))
    }

    pub fn value(name: &str, value: impl Into<AnyValue<'static>>) -> Self {
        let inner = value.into();
        Param(Arc::new(name.to_string()), inner)
    }

    pub fn empty() -> Self {
        Param(Arc::new("".to_string()), AnyValue::Null)
    }

    pub fn add_value(mut self, name: &str, value: impl Into<AnyValue<'static>>) -> Self {
        let inner = value.into();
        if let AnyValue::Struct(ref mut fields) = self.1 {
            let dtype = inner.dtype();
            let field = Field::new(name.into(), dtype.clone());
            fields.push((field, inner));
        } else {
            panic!("Cannot add field to non-struct parameter");
        }
        self
    }

    pub fn add_field(&mut self, name: &str, value: impl Into<AnyValue<'static>>) -> &mut Self {
        let inner = value.into();
        if let AnyValue::Struct(ref mut fields) = self.1 {
            let dtype = inner.dtype();
            let field = Field::new(name.into(), dtype.clone());
            fields.push((field, inner));
        } else {
            panic!("Cannot add field to non-struct parameter");
        }

        self
    }
}

impl Default for Param {
    fn default() -> Self {
        Param::empty()
    }
}
