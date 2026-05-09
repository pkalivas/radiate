use radiate_utils::{AnyValue, DataType, Field, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Implemented by types that can describe themselves as a [`Frozen`].
///
/// Component traits like `Select`, `Crossover`, `Mutate`, `Codec`, etc. each
/// have their own `freeze()` method on the trait surface. Hand-rolled
/// implementations build a `Frozen` directly; `Freezable` is the layer the
/// `#[derive(Freeze)]` macro targets, and concrete trait impls can delegate to
/// it via `<Self as Freezable>::as_frozen(self)`.
pub trait Freezable {
    fn as_frozen(&self) -> Frozen;
}

/// Re-export of the `#[derive(Freeze)]` proc-macro from `radiate-derive`.
pub use radiate_derive::Freeze;

/// A read-only snapshot of an engine's configurable knobs at `build()` time.
///
/// Keyed by user-supplied labels (e.g. `"offspring_selector"`), each entry is
/// any [`AnyValue`] — typically a [`Frozen`] (`AnyValue::Map`) describing a
/// component, but can also be a sequence (`AnyValue::Vector`, e.g. `alters`)
/// or a scalar.
#[derive(Debug, Clone, Default)]
pub struct Freeze {
    entries: BTreeMap<String, AnyValue<'static>>,
}

impl Freeze {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a frozen entry using its embedded `type` field as the key (when
    /// the value is a [`Frozen`] with one). Falls back to a positional key.
    pub fn register(&mut self, entry: Frozen) {
        let key = entry
            .type_name()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("entry_{}", self.entries.len()));
        self.insert(key, entry);
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<AnyValue<'static>>) {
        self.entries.insert(name.into(), value.into());
    }

    pub fn get(&self, name: &str) -> Option<&AnyValue<'static>> {
        self.entries.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut AnyValue<'static>> {
        self.entries.get_mut(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &AnyValue<'static>)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The value of an entry's `"type"` field, if the entry is a frozen map
    /// that carries one. Useful for type-tag introspection without importing
    /// `AnyValue`.
    pub fn type_name(&self, name: &str) -> Option<&str> {
        match self.entries.get(name)? {
            AnyValue::Map(fields) => fields
                .iter()
                .find_map(|(n, _, v)| (n.as_str() == "type").then(|| v.as_str()).flatten()),
            _ => None,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Freeze {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for (key, value) in &self.entries {
            map.serialize_entry(key, &FrozenValue(value))?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Freeze {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw: BTreeMap<String, ScalarValue> = BTreeMap::deserialize(deserializer)?;
        let entries = raw.into_iter().map(|(k, v)| (k, v.into())).collect();
        Ok(Freeze { entries })
    }
}

fn short_type_name<T: ?Sized>() -> String {
    // Walk the full type name and strip the module path of every segment, not
    // just the head. Segments are delimited by `<`, `>`, `,`, or space; each
    // segment becomes whatever follows its last `::`. So
    // `module::Foo<other::Bar<u8>>` renders as `Foo<Bar<u8>>`.
    let full = std::any::type_name::<T>();
    let mut out = String::with_capacity(full.len());
    let mut segment_start = 0usize;
    for (i, c) in full.char_indices() {
        if matches!(c, '<' | '>' | ',' | ' ') {
            out.push_str(strip_path(&full[segment_start..i]));
            out.push(c);
            segment_start = i + c.len_utf8();
        }
    }
    if segment_start < full.len() {
        out.push_str(strip_path(&full[segment_start..]));
    }
    out
}

fn strip_path(segment: &str) -> &str {
    match segment.rfind("::") {
        Some(idx) => &segment[idx + 2..],
        None => segment,
    }
}

/// Ergonomic builder for `AnyValue::Map`. Mirrors the `Frozen::typed::<T>()
/// .with(name, value)` shape but lands in `AnyValue` directly.
#[derive(Debug, Default, Clone)]
pub struct Frozen {
    fields: Vec<(SmallStr, DataType, AnyValue<'static>)>,
}

impl Frozen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: impl Into<String>) -> Self {
        let mut freeze = Self::new();
        freeze
            .fields
            .push((SmallStr::from(name.into()), DataType::Null, AnyValue::Null));
        freeze
    }

    pub fn value(value: impl Into<AnyValue<'static>>) -> Self {
        Self::new().with_value(value)
    }

    /// Builder seeded with a `("type", <short type name>)` entry, matching the
    /// previous `Frozen::typed::<T>()` convention.
    pub fn typed<T: ?Sized>() -> Self {
        Self::new().with("type", short_type_name::<T>())
    }

    pub fn with(mut self, name: impl Into<SmallStr>, value: impl Into<AnyValue<'static>>) -> Self {
        let value = value.into();
        let dtype = value.dtype();
        self.fields.push((name.into(), dtype, value));
        self
    }

    pub fn with_value(self, value: impl Into<AnyValue<'static>>) -> Self {
        self.with("value", value)
    }

    pub fn build(self) -> AnyValue<'static> {
        AnyValue::Map(self.fields)
    }

    /// The value of the conventional `"type"` field, if present.
    pub fn type_name(&self) -> Option<&str> {
        self.fields.iter().find_map(|(name, _, value)| {
            (name.as_str() == "type").then(|| value.as_str()).flatten()
        })
    }
}

impl From<Frozen> for AnyValue<'static> {
    fn from(b: Frozen) -> Self {
        b.build()
    }
}

impl FromIterator<Frozen> for Frozen {
    fn from_iter<T: IntoIterator<Item = Frozen>>(iter: T) -> Self {
        let fields = iter
            .into_iter()
            .map(|f| {
                let name = f
                    .type_name()
                    .map(SmallStr::from)
                    .unwrap_or_else(|| SmallStr::from("entry"));
                let value = f.build();
                let dtype = value.dtype();
                (name, dtype, value)
            })
            .collect();

        Self { fields }
    }
}

/// Build a structured frozen entry for a `std::ops::Range<T>` where `T` is a
/// numeric type that can be cast to `f64`. Renders as `{start: ..., end: ...}`.
pub fn frozen_range<T: num_traits::NumCast + Copy>(range: &std::ops::Range<T>) -> Frozen {
    let start: f64 = num_traits::cast(range.start).unwrap_or(f64::NAN);
    let end: f64 = num_traits::cast(range.end).unwrap_or(f64::NAN);
    Frozen::new().with("start", start).with("end", end)
}

#[cfg(feature = "serde")]
impl Serialize for Frozen {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (name, _dtype, value) in &self.fields {
            map.serialize_entry(name.as_str(), value)?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Frozen {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FrozenVisitor;

        impl<'de> serde::de::Visitor<'de> for FrozenVisitor {
            type Value = Frozen;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a frozen map")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Frozen, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut fields = Vec::with_capacity(access.size_hint().unwrap_or(0));
                while let Some(name) = access.next_key::<String>()? {
                    let any: AnyValue<'static> = access.next_value::<ScalarValue>()?.into();
                    let dtype = any.dtype();
                    fields.push((SmallStr::from(name), dtype, any));
                }
                Ok(Frozen { fields })
            }
        }

        deserializer.deserialize_map(FrozenVisitor)
    }
}

/// Wire-format serializer for a `Frozen` inside a `Freeze`. When the entry has
/// a `"type"` field, emits it as `{<type>: {body without type}}` (one extra
/// Smart-serialize wrapper for an `AnyValue` field. For `AnyValue::Map`, it
/// renders the map as a clean YAML/JSON object — hoisting any `"type"` field
/// up to be the outer key — and recurses into nested maps. For sequences, it
/// recurses element-wise. For all other variants, it falls through to
/// `AnyValue`'s default `Serialize` impl.
#[cfg(feature = "serde")]
struct FrozenValue<'a>(&'a AnyValue<'static>);

#[cfg(feature = "serde")]
impl<'a> Serialize for FrozenValue<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeMap, SerializeSeq};
        match self.0 {
            AnyValue::Map(entries) => {
                let type_name = entries
                    .iter()
                    .find_map(|(n, _, v)| (n.as_str() == "type").then(|| v.as_str()).flatten());
                if let Some(tn) = type_name {
                    let mut m = serializer.serialize_map(Some(1))?;
                    m.serialize_entry(tn, &FrozenMapBody(entries))?;
                    m.end()
                } else {
                    FrozenMapBody(entries).serialize(serializer)
                }
            }
            AnyValue::Vector(vals) => {
                let mut seq = serializer.serialize_seq(Some(vals.len()))?;
                for v in vals {
                    seq.serialize_element(&FrozenValue(v))?;
                }
                seq.end()
            }
            AnyValue::Slice(vals) => {
                let mut seq = serializer.serialize_seq(Some(vals.len()))?;
                for v in *vals {
                    seq.serialize_element(&FrozenValue(v))?;
                }
                seq.end()
            }
            other => other.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
struct FrozenMapBody<'a>(&'a [(SmallStr, DataType, AnyValue<'static>)]);

#[cfg(feature = "serde")]
impl<'a> Serialize for FrozenMapBody<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let len = self
            .0
            .iter()
            .filter(|(n, _, _)| n.as_str() != "type")
            .count();
        let mut m = serializer.serialize_map(Some(len))?;
        for (name, _, value) in self.0 {
            if name.as_str() == "type" {
                continue;
            }
            m.serialize_entry(name.as_str(), &FrozenValue(value))?;
        }
        m.end()
    }
}

/// Inferred scalar shape for a scalar value coming from a self-describing
/// format like YAML/JSON. The dtype is recovered from the Rust type at parse
/// time, so the on-disk representation can be a plain scalar (no tag).
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(untagged)]
enum ScalarValue {
    Bool(bool),
    UInt(u64),
    Int(i64),
    Float(f64),
    String(String),
    Vec(Vec<ScalarValue>),
    Map(BTreeMap<String, ScalarValue>),
}

#[cfg(feature = "serde")]
impl From<ScalarValue> for AnyValue<'static> {
    fn from(v: ScalarValue) -> Self {
        match v {
            ScalarValue::Bool(b) => AnyValue::Bool(b),
            ScalarValue::UInt(u) if u <= usize::MAX as u64 => AnyValue::Usize(u as usize),
            ScalarValue::UInt(u) => AnyValue::UInt64(u),
            ScalarValue::Int(i) => AnyValue::Int64(i),
            ScalarValue::Float(f) => AnyValue::Float64(f),
            ScalarValue::String(s) => AnyValue::StrOwned(s),
            ScalarValue::Vec(vs) => AnyValue::Vector(vs.into_iter().map(Into::into).collect()),
            ScalarValue::Map(m) => {
                let mut fields: Vec<(SmallStr, DataType, AnyValue<'static>)> = m
                    .into_iter()
                    .map(|(k, v)| {
                        let any: AnyValue<'static> = v.into();
                        (SmallStr::from(k), any.dtype(), any)
                    })
                    .collect();

                // Un-hoist the serialized nested form: a one-entry map whose
                // value is itself a map (and that inner map doesn't already
                // carry its own `"type"` field) is the result of `FrozenWire`
                // hoisting a type tag up to be the outer key. Reverse it so
                // the in-memory shape is `{type, ...body}` again.
                if fields.len() == 1
                    && matches!(&fields[0].2, AnyValue::Map(inner)
                        if !inner.iter().any(|(n, _, _)| n.as_str() == "type"))
                {
                    let (key, _, inner) = fields.pop().unwrap();
                    let AnyValue::Map(inner_fields) = inner else {
                        unreachable!()
                    };
                    let mut new_fields = Vec::with_capacity(inner_fields.len() + 1);
                    new_fields.push((
                        SmallStr::from("type"),
                        DataType::String,
                        AnyValue::StrOwned(key.to_string()),
                    ));
                    new_fields.extend(inner_fields);
                    return AnyValue::Map(new_fields);
                }

                AnyValue::Map(fields)
            }
        }
    }
}

/// Build a named `AnyValue::Struct` (rather than an anonymous `Map`). Use this
/// when the outer name carries meaning — e.g. wrapping a selector's frozen state with
/// the selector's type as the struct name. For most freeze use cases,
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
mod tests {
    use super::*;

    #[test]
    fn map_builder_typed_emits_type_and_fields_in_order() {
        struct Example;
        let v = Frozen::typed::<Example>().with("k", 3usize).build();

        match v {
            AnyValue::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0.as_str(), "type");
                assert_eq!(entries[0].2.as_str(), Some("Example"));
                assert_eq!(entries[1].0.as_str(), "k");
                assert!(matches!(&entries[1].2, AnyValue::Usize(3)));
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }

    #[test]
    fn map_builder_nests_via_into_anyvalue() {
        let inner = Frozen::new().with("temperature", 4.0f32);
        let outer = Frozen::new()
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

    #[cfg(feature = "serde")]
    #[test]
    fn freeze_round_trips_with_label_and_hoisted_type() {
        let mut set = Freeze::default();
        set.insert(
            "offspring_selector",
            Frozen::new()
                .with("type", "BoltzmannSelector")
                .with("temperature", 4.0f32),
        );
        set.insert(
            "survivor_selector",
            Frozen::new()
                .with("type", "TournamentSelector")
                .with("k", 3usize),
        );

        let yaml = yaml_serde::to_string(&set).unwrap();
        let expected = "offspring_selector:\n  BoltzmannSelector:\n    temperature: 4.0\nsurvivor_selector:\n  TournamentSelector:\n    k: 3\n";
        assert_eq!(yaml, expected);

        let round_tripped: Freeze = yaml_serde::from_str(&yaml).unwrap();
        assert_eq!(round_tripped.len(), 2);
        assert_eq!(
            round_tripped.type_name("offspring_selector"),
            Some("BoltzmannSelector"),
        );
        assert_eq!(
            round_tripped.type_name("survivor_selector"),
            Some("TournamentSelector"),
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn frozen_with_nested_field_round_trips() {
        // A frozen entry whose values include another nested frozen — like a
        // crossover with a structured `rate` schedule.
        let mut set = Freeze::default();
        set.insert(
            "alterer_0",
            Frozen::new()
                .with("type", "MultiPointCrossover")
                .with(
                    "rate",
                    Frozen::new()
                        .with("type", "Linear")
                        .with("start", 0.1f32)
                        .with("end", 0.9f32)
                        .with("steps", 100usize),
                )
                .with("num_points", 2usize),
        );

        let yaml = yaml_serde::to_string(&set).unwrap();
        let expected = "alterer_0:\n  MultiPointCrossover:\n    rate:\n      Linear:\n        start: 0.1\n        end: 0.9\n        steps: 100\n    num_points: 2\n";
        assert_eq!(yaml, expected);

        let round: Freeze = yaml_serde::from_str(&yaml).unwrap();
        assert_eq!(round.type_name("alterer_0"), Some("MultiPointCrossover"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn frozen_with_sequence_field_round_trips() {
        // A frozen entry whose value is a Vec of nested frozens — like a Range
        // pair list or an Objective::Multi labels list.
        let entries: Vec<AnyValue<'static>> = vec![
            Frozen::new()
                .with("step", 0usize)
                .with("rate", 0.5f32)
                .build(),
            Frozen::new()
                .with("step", 10usize)
                .with("rate", 0.3f32)
                .build(),
        ];
        let mut set = Freeze::default();
        set.insert(
            "schedule",
            Frozen::new()
                .with("type", "Stepwise")
                .with("steps", AnyValue::Vector(entries)),
        );

        let yaml = yaml_serde::to_string(&set).unwrap();
        let expected = "schedule:\n  Stepwise:\n    steps:\n    - step: 0\n      rate: 0.5\n    - step: 10\n      rate: 0.3\n";
        assert_eq!(yaml, expected);

        let round: Freeze = yaml_serde::from_str(&yaml).unwrap();
        assert_eq!(round.type_name("schedule"), Some("Stepwise"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn freeze_with_alters_list_round_trips() {
        // The headline win: alters render as a single YAML list under one key,
        // not as `alterer_0`, `alterer_1`, ...
        let alters: Vec<AnyValue<'static>> = vec![
            Frozen::new()
                .with("type", "MeanCrossover")
                .with("rate", 0.5f32)
                .build(),
            Frozen::new()
                .with("type", "GaussianMutator")
                .with("rate", 0.1f32)
                .build(),
        ];
        let mut set = Freeze::default();
        set.insert("alters", AnyValue::Vector(alters));

        let yaml = yaml_serde::to_string(&set).unwrap();
        let expected =
            "alters:\n- MeanCrossover:\n    rate: 0.5\n- GaussianMutator:\n    rate: 0.1\n";
        assert_eq!(yaml, expected);

        let round: Freeze = yaml_serde::from_str(&yaml).unwrap();
        let entry = round.get("alters").unwrap();
        let AnyValue::Vector(items) = entry else {
            panic!("expected Vector, got {:?}", entry);
        };
        assert_eq!(items.len(), 2);
        // First alter should round-trip with `type` un-hoisted back into the body.
        let AnyValue::Map(first) = &items[0] else {
            panic!("expected Map");
        };
        assert!(
            first
                .iter()
                .any(|(n, _, v)| n.as_str() == "type" && v.as_str() == Some("MeanCrossover"))
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn freeze_with_selectors_submap_round_trips() {
        // Selectors live under a single `selectors:` key with `offspring` and
        // `survivor` as inner sub-frozen entries — same shape as the engine
        // builder produces.
        let mut set = Freeze::default();
        set.insert(
            "selectors",
            Frozen::new()
                .with(
                    "offspring",
                    Frozen::new()
                        .with("type", "BoltzmannSelector")
                        .with("temperature", 4.0f32),
                )
                .with(
                    "survivor",
                    Frozen::new()
                        .with("type", "TournamentSelector")
                        .with("k", 3usize),
                ),
        );

        let yaml = yaml_serde::to_string(&set).unwrap();
        let expected = "selectors:\n  offspring:\n    BoltzmannSelector:\n      temperature: 4.0\n  survivor:\n    TournamentSelector:\n      k: 3\n";
        assert_eq!(yaml, expected);

        let round: Freeze = yaml_serde::from_str(&yaml).unwrap();
        let AnyValue::Map(top) = round.get("selectors").unwrap() else {
            panic!("expected selectors to be a Map");
        };
        let off = top
            .iter()
            .find(|(n, _, _)| n.as_str() == "offspring")
            .expect("offspring entry");
        let AnyValue::Map(off_fields) = &off.2 else {
            panic!("expected offspring to be a Map");
        };
        assert!(
            off_fields
                .iter()
                .any(|(n, _, v)| n.as_str() == "type" && v.as_str() == Some("BoltzmannSelector"))
        );
        let surv = top
            .iter()
            .find(|(n, _, _)| n.as_str() == "survivor")
            .expect("survivor entry");
        let AnyValue::Map(surv_fields) = &surv.2 else {
            panic!("expected survivor to be a Map");
        };
        assert!(
            surv_fields
                .iter()
                .any(|(n, _, v)| n.as_str() == "type" && v.as_str() == Some("TournamentSelector"))
        );
    }
}
