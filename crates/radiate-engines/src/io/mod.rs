#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

pub trait FileWriter<T> {
    fn extension(&self) -> &str;
    fn write(&mut self, path: PathBuf, generation: &T) -> io::Result<()>;
}

pub trait FileReader<T> {
    fn read(&self, path: PathBuf) -> io::Result<T>;
}

pub struct JsonWriter;

#[cfg(feature = "serde")]
impl<T> FileWriter<T> for JsonWriter
where
    T: Serialize,
{
    fn extension(&self) -> &str {
        "json"
    }

    fn write(&mut self, path: PathBuf, generation: &T) -> io::Result<()> {
        if !path.parent().is_none_or(|p| p.exists()) {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
                io::Error::other(format!("Failed to create checkpoint directory: {}", e))
            })?;
        }

        let json = serde_json::to_string(generation)
            .map_err(|e| io::Error::other(format!("Failed to serialize checkpoint file: {}", e)))?;

        std::fs::write(path, json)
    }
}

pub struct JsonReader;

#[cfg(feature = "serde")]
impl<T> FileReader<T> for JsonReader
where
    T: for<'de> Deserialize<'de>,
{
    fn read(&self, path: PathBuf) -> io::Result<T> {
        let json = std::fs::read_to_string(path)?;
        let generation = serde_json::from_str(&json).map_err(|e| {
            io::Error::other(format!("Failed to deserialize checkpoint file: {}", e))
        })?;

        Ok(generation)
    }
}

pub struct YamlWriter;

#[cfg(feature = "serde")]
impl<T> FileWriter<T> for YamlWriter
where
    T: Serialize,
{
    fn extension(&self) -> &str {
        "yaml"
    }

    fn write(&mut self, path: PathBuf, generation: &T) -> io::Result<()> {
        if !path.parent().is_none_or(|p| p.exists()) {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
                io::Error::other(format!("Failed to create checkpoint directory: {}", e))
            })?;
        }

        let yaml = yaml_serde::to_string(generation)
            .map_err(|e| io::Error::other(format!("Failed to serialize checkpoint file: {}", e)))?;

        std::fs::write(path, yaml)
    }
}

pub struct YamlReader;

#[cfg(feature = "serde")]
impl<T> FileReader<T> for YamlReader
where
    T: for<'de> Deserialize<'de>,
{
    fn read(&self, path: PathBuf) -> io::Result<T> {
        let yaml = std::fs::read_to_string(path)?;
        let generation = yaml_serde::from_str(&yaml).map_err(|e| {
            io::Error::other(format!("Failed to deserialize checkpoint file: {}", e))
        })?;

        Ok(generation)
    }
}
