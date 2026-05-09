use radiate::{FileReader, FileWriter};
use serde_pickle::{DeOptions, SerOptions};
use std::path::PathBuf;

pub struct PickleWriter;

impl<T> FileWriter<T> for PickleWriter
where
    T: serde::Serialize,
{
    fn extension(&self) -> &str {
        "pkl"
    }

    fn write(&mut self, path: PathBuf, generation: &T) -> std::io::Result<()> {
        if !path.parent().map_or(true, |p| p.exists()) {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
                std::io::Error::other(format!("Failed to create checkpoint directory: {}", e))
            })?;
        }
        let pickle = serde_pickle::to_vec(generation, SerOptions::default()).map_err(|e| {
            std::io::Error::other(format!("Failed to serialize checkpoint to pickle: {}", e))
        })?;
        std::fs::write(path, pickle)
    }
}

pub struct PickleReader;

impl<T> FileReader<T> for PickleReader
where
    T: for<'de> serde::de::DeserializeOwned,
{
    fn read(&self, path: PathBuf) -> std::io::Result<T> {
        let pickle = std::fs::read(path)?;
        let generation = serde_pickle::from_slice(&pickle, DeOptions::default()).map_err(|e| {
            std::io::Error::other(format!(
                "Failed to deserialize checkpoint from pickle: {}",
                e
            ))
        })?;

        Ok(generation)
    }
}
