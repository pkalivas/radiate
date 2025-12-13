use crate::any::{time_unit, time_zone};
use crate::{AnyChromosome, AnyGene, AnyValue, Field};
use radiate::{Chromosome, Gene};
use serde::ser::Serializer;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
enum AnyValueSerializable {
    #[default]
    Null,
    Bool(bool),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Float32(f32),
    Float64(f64),
    Binary(Vec<u8>),
    Char(char),
    Str(String),
    StrOwned(String),
    Date(i32),
    DateTime(i64, time_unit::TimeUnit, Option<time_zone::TimeZone>),
    Vector(Box<Vec<AnyValueSerializable>>),
    Struct(Vec<(Field, AnyValueSerializable)>),
}

impl AnyValueSerializable {
    pub fn into_static(self) -> AnyValue<'static> {
        match self {
            AnyValueSerializable::Null => AnyValue::Null,
            AnyValueSerializable::Bool(b) => AnyValue::Bool(b),
            AnyValueSerializable::UInt8(u) => AnyValue::UInt8(u),
            AnyValueSerializable::UInt16(u) => AnyValue::UInt16(u),
            AnyValueSerializable::UInt32(u) => AnyValue::UInt32(u),
            AnyValueSerializable::UInt64(u) => AnyValue::UInt64(u),
            AnyValueSerializable::Int8(i) => AnyValue::Int8(i),
            AnyValueSerializable::Int16(i) => AnyValue::Int16(i),
            AnyValueSerializable::Int32(i) => AnyValue::Int32(i),
            AnyValueSerializable::Int64(i) => AnyValue::Int64(i),
            AnyValueSerializable::Int128(i) => AnyValue::Int128(i),
            AnyValueSerializable::Float32(f) => AnyValue::Float32(f),
            AnyValueSerializable::Float64(f) => AnyValue::Float64(f),
            AnyValueSerializable::Binary(b) => AnyValue::Binary(b),
            AnyValueSerializable::Char(c) => AnyValue::Char(c),
            AnyValueSerializable::Str(s) => AnyValue::StrOwned(s),
            AnyValueSerializable::StrOwned(s) => AnyValue::StrOwned(s),
            AnyValueSerializable::Date(d) => AnyValue::Date(d),
            AnyValueSerializable::DateTime(v, tu, tz) => {
                AnyValue::DateTime(v, tu, tz.map(|t| Arc::new(t)))
            }
            AnyValueSerializable::Vector(v) => {
                let vec = v.into_iter().map(|av| av.into_static()).collect();
                AnyValue::Vector(Box::new(vec))
            }
            AnyValueSerializable::Struct(fields) => {
                let struct_fields = fields
                    .into_iter()
                    .map(|(f, av)| (f, av.into_static()))
                    .collect();
                AnyValue::Struct(struct_fields)
            }
        }
    }
}

// This stays basically as you intended, but using the helper struct is nicer:
impl Serialize for AnyGene<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct AnyGeneRepr {
            allele: AnyValue<'static>,
            metadata: Option<HashMap<String, String>>,
        }

        let repr = AnyGeneRepr {
            allele: self.allele().clone().into_static(),
            metadata: self.metadata().map(|m| m.clone()),
        };

        repr.serialize(serializer)
    }
}

impl<'de, 'a> Deserialize<'de> for AnyGene<'static>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AnyGeneRepr {
            allele: AnyValueSerializable,
            metadata: Option<HashMap<String, String>>,
        }

        let repr = AnyGeneRepr::deserialize(deserializer)?;

        Ok(
            AnyGene::new(repr.allele.into_static())
                .with_metadata(repr.metadata.unwrap_or_default()),
        )
    }
}

impl Serialize for AnyChromosome<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.genes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyChromosome<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let genes: Vec<AnyGene<'static>> = Deserialize::deserialize(deserializer)?;
        Ok(AnyChromosome::new(genes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gene_serialization() {
        let gene = AnyGene::new(AnyValue::Int32(42)).with_metadata(HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]));

        let serialized = serde_json::to_string(&gene).unwrap();
        let deserialized: AnyGene<'static> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(gene.allele(), deserialized.allele());
        assert_eq!(gene.metadata(), deserialized.metadata());
    }

    #[test]
    fn test_chromosome_serialization() {
        let gene1 = AnyGene::new(AnyValue::Int32(42));
        let gene2 = AnyGene::new(AnyValue::StrOwned("Hello".to_string()));
        let chromosome = AnyChromosome::new(vec![gene1, gene2]);

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: AnyChromosome<'static> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.genes().len(), deserialized.genes().len());
    }
}
