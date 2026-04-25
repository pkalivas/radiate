use radiate::{CheckpointReader, CheckpointWriter, Chromosome, Generation};
use serde_pickle::{DeOptions, SerOptions};
use std::path::PathBuf;

pub struct PickleCheckpointWriter;

impl<C, T> CheckpointWriter<C, T> for PickleCheckpointWriter
where
    C: Chromosome + serde::Serialize,
    T: serde::Serialize,
{
    fn extension(&self) -> &str {
        "pkl"
    }

    fn write_checkpoint(
        &mut self,
        path: PathBuf,
        generation: &Generation<C, T>,
    ) -> std::io::Result<()> {
        let pickle = serde_pickle::to_vec(generation, SerOptions::default()).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to serialize checkpoint to pickle: {}", e),
            )
        })?;

        std::fs::write(path, pickle)
    }
}

pub struct PickleCheckpointReader;

impl<C, T> CheckpointReader<C, T> for PickleCheckpointReader
where
    C: Chromosome + for<'de> serde::de::DeserializeOwned,
    T: for<'de> serde::de::DeserializeOwned,
{
    fn read_checkpoint(&self, path: PathBuf) -> std::io::Result<Generation<C, T>> {
        let pickle = std::fs::read(path)?;
        let generation = serde_pickle::from_slice(&pickle, DeOptions::default()).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to deserialize checkpoint from pickle: {}", e),
            )
        })?;

        Ok(generation)
    }
}
