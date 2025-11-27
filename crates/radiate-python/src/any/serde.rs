use crate::{AnyChromosome, AnyGene, AnyValue};
use radiate::{Chromosome, Gene};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::sync::Arc;

impl Serialize for AnyGene<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AnyGene", 2)?;
        state.serialize_field("allele", &self.allele().clone().into_static())?;
        state.serialize_field("metadata", &self.metadata())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AnyGene<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let gene = serde_json::Value::deserialize(deserializer)?;

        let allele = gene
            .get("allele")
            .ok_or_else(|| serde::de::Error::missing_field("allele"))?;
        let serde_string = match allele {
            serde_json::Value::String(s) => s.clone(),
            _ => {
                return Err(serde::de::Error::custom(
                    "Expected allele to be a serialized string",
                ));
            }
        };

        let allele = serde_json::from_str::<AnyValue<'_>>(&serde_string).map_err(|e| {
            serde::de::Error::custom(format!("Failed to deserialize allele: {}", e))
        })?;

        let metadata = gene
            .get("metadata")
            .ok_or_else(|| serde::de::Error::missing_field("metadata"))?;
        let metadata =
            serde_json::from_value::<Option<Arc<HashMap<String, String>>>>(metadata.clone())
                .map_err(|e| {
                    serde::de::Error::custom(format!("Failed to deserialize metadata: {}", e))
                })?;

        let gene = AnyGene::new(allele.into_static());
        if let Some(meta) = metadata {
            Ok(gene.with_metadata((*meta).clone()))
        } else {
            Ok(gene)
        }
    }
}

impl Serialize for AnyChromosome<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AnyChromosome", 1)?;
        state.serialize_field("genes", self.genes())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AnyChromosome<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let chromo = serde_json::Value::deserialize(deserializer)?;

        let genes = chromo
            .get("genes")
            .ok_or_else(|| serde::de::Error::missing_field("genes"))?;
        let genes: Vec<AnyGene<'static>> = serde_json::from_value(genes.clone())
            .map_err(|e| serde::de::Error::custom(format!("Failed to deserialize genes: {}", e)))?;

        Ok(AnyChromosome::new(genes))
    }
}
