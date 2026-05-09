use std::{collections::BTreeMap, sync::Arc};

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
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone)]
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

// =============================================================================
// AnyValue-based alternative to Parameter / ParameterSet
// =============================================================================
//
// `Parameter` and `ParameterSet` above are functionally a thin layer over
// `AnyValue::Map` — a keyed bag of values that recurses for nested groups.
// `MapBuilder` below produces `AnyValue::Map` directly with the same
// `typed::<T>().with(name, value)` ergonomics, so selectors can drop the
// dependency on `Parameter` entirely.
//
// Migration sketch (selector trait):
//
//     // before
//     fn params(&self) -> Parameter {
//         Parameter::typed::<Self>().with("k", self.k)
//     }
//
//     // after
//     fn params(&self) -> AnyValue<'static> {
//         MapBuilder::typed::<Self>().with("k", self.k).build()
//     }
//
// Migration sketch (engine-level parameter set):
//
//     // before
//     pub parameter_set: ParameterSet
//
//     // after
//     pub parameter_set: AnyValue<'static>     // built via MapBuilder
//
// Wins:
//   - one less custom type, one less serde surface (AnyValue already handles it)
//   - parameter trees compose with `radiate-expr` (SelectExpr / projections)
//     without any glue
//   - insertion order preserved (AnyValue::Map is `Vec<(Field, _)>`,
//     unlike Parameter::Section's BTreeMap)
//   - no `#[serde(untagged)]` ambiguity

use radiate_utils::{DataType, Field, SmallStr};

/// Ergonomic builder for `AnyValue::Map`. Mirrors the `Parameter::typed::<T>()
/// .with(name, value)` shape but lands in `AnyValue` directly.
#[derive(Debug, Default, Clone)]
pub struct MapBuilder {
    fields: Vec<(Arc<String>, DataType, AnyValue<'static>)>,
}

impl MapBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder seeded with a `("type", <short type name>)` entry, matching the
    /// previous `Parameter::typed::<T>()` convention.
    pub fn typed<T: ?Sized>() -> Self {
        Self::new().with("type", short_type_name::<T>())
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<AnyValue<'static>>) -> Self {
        let value = value.into();
        let dtype = value.dtype();
        let name_str = name.into();
        self.fields
            .push((radiate_utils::cache_arc_string!(name_str), dtype, value));
        self
    }

    pub fn build(self) -> AnyValue<'static> {
        AnyValue::Map(self.fields)
    }
}

impl From<MapBuilder> for AnyValue<'static> {
    fn from(b: MapBuilder) -> Self {
        b.build()
    }
}

/// Build a named `AnyValue::Struct` (rather than an anonymous `Map`). Use this
/// when the outer name carries meaning — e.g. wrapping a selector's params with
/// the selector's type as the struct name. For most parameter use cases,
/// `MapBuilder::typed::<T>()` is simpler.
#[derive(Debug, Default, Clone)]
pub struct StructBuilder {
    name: SmallStr,
    fields: Vec<(Field, AnyValue<'static>)>,
}

impl StructBuilder {
    pub fn new(name: impl Into<SmallStr>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    pub fn typed<T: ?Sized>() -> Self {
        Self::new(short_type_name::<T>())
    }

    pub fn with(mut self, name: impl Into<SmallStr>, value: impl Into<AnyValue<'static>>) -> Self {
        let value = value.into();
        let dtype = value.dtype();
        self.fields.push((Field::new(name.into(), dtype), value));
        self
    }

    pub fn build(self) -> AnyValue<'static> {
        // Outer Field carries the struct's name. Its dtype is `DataType::Null`
        // as a placeholder — the accurate `DataType::Struct(...)` is recoverable
        // by calling `.dtype()` on the resulting AnyValue.
        let outer = Field::new(self.name, DataType::Null);
        AnyValue::Struct(outer, self.fields)
    }
}

impl From<StructBuilder> for AnyValue<'static> {
    fn from(b: StructBuilder) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod any_value_param_tests {
    use super::*;

    #[test]
    fn map_builder_typed_emits_type_and_fields_in_order() {
        struct Example;
        let v = MapBuilder::typed::<Example>().with("k", 3usize).build();

        println!("{:#?}", v);

        match v {
            AnyValue::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0.as_str(), "type");
                assert!(matches!(&entries[0].2, AnyValue::Str(s) if *s == "Example"));
                assert_eq!(entries[1].0.as_str(), "k");
                assert!(matches!(&entries[1].2, AnyValue::Usize(3)));
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }

    #[test]
    fn map_builder_nests_via_into_anyvalue() {
        let inner = MapBuilder::new().with("temperature", 4.0f32);
        let outer = MapBuilder::new()
            .with("offspring_selector", inner) // MapBuilder: Into<AnyValue>
            .build();

        match outer {
            AnyValue::Map(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0.as_str(), "offspring_selector");
                assert!(matches!(&entries[0].2, AnyValue::Map(_)));
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }
}
