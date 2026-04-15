use crate::Generation;
use radiate_core::Chromosome;
#[cfg(feature = "serde")]
use serde::Deserialize;
use std::path::PathBuf;

pub trait CheckpointWriter<C, T>
where
    C: Chromosome,
{
    fn extension(&self) -> &str;
    fn write_checkpoint(
        &mut self,
        path: PathBuf,
        generation: &Generation<C, T>,
    ) -> std::io::Result<()>;
}

pub trait CheckpointReader<C, T>
where
    C: Chromosome,
{
    fn read_checkpoint(&self, path: PathBuf) -> std::io::Result<Generation<C, T>>;
}

pub struct JsonCheckpointWriter;

#[cfg(feature = "serde")]
impl<C, T> CheckpointWriter<C, T> for JsonCheckpointWriter
where
    C: Chromosome + serde::Serialize,
    T: serde::Serialize,
{
    fn extension(&self) -> &str {
        "json"
    }

    fn write_checkpoint(
        &mut self,
        path: PathBuf,
        generation: &Generation<C, T>,
    ) -> std::io::Result<()> {
        let json = serde_json::to_string(generation).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to serialize checkpoint file: {}", e),
            )
        })?;

        std::fs::write(path, json)
    }
}

pub struct JsonCheckpointReader;

#[cfg(feature = "serde")]
impl<C, T> CheckpointReader<C, T> for JsonCheckpointReader
where
    C: Chromosome + for<'de> Deserialize<'de>,
    T: for<'de> Deserialize<'de>,
{
    fn read_checkpoint(&self, path: PathBuf) -> std::io::Result<Generation<C, T>> {
        let json = std::fs::read_to_string(path)?;
        let generation = serde_json::from_str(&json).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to deserialize checkpoint file: {}", e),
            )
        })?;

        Ok(generation)
    }
}
