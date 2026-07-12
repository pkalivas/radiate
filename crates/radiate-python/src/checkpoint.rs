use crate::{EpochHandle, PyGeneration};
use pyo3::{
    Python,
    types::{PyBytes, PyBytesMethods},
};
use radiate::{Chromosome, FileReader, FileWriter, Generation};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

const PICKLE_EXTENSION: &str = "pkl";
const JSON_EXTENSION: &str = "json";

pub struct PyCheckpointWriter(pub String);

impl<C: Chromosome, T> FileWriter<Generation<C, T>> for PyCheckpointWriter
where
    Generation<C, T>: serde::Serialize + Into<EpochHandle>,
    C: Chromosome + Clone,
    T: Clone,
{
    fn extension(&self) -> &str {
        &self.0
    }

    fn write(&mut self, path: PathBuf, generation: &Generation<C, T>) -> std::io::Result<()> {
        if !path.parent().is_none_or(|p| p.exists()) {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
                std::io::Error::other(format!("Failed to create checkpoint directory: {}", e))
            })?;
        }

        Python::attach(|py| {
            let py_generation = PyGeneration::new(generation.clone().into());

            match self.0.as_str() {
                JSON_EXTENSION => {
                    let json_str = py_generation.to_json()?;
                    std::fs::write(path, json_str)
                }
                PICKLE_EXTENSION => {
                    let pickle_bytes = py_generation.to_pickle(py)?;
                    std::fs::write(path, pickle_bytes.as_bytes())
                }
                _ => Err(std::io::Error::other(format!(
                    "Unsupported checkpoint format: {}",
                    self.0
                ))),
            }
        })
    }
}

pub struct PyCheckpointReader(pub String);

impl<C: Chromosome, T> FileReader<Generation<C, T>> for PyCheckpointReader
where
    Generation<C, T>: for<'de> DeserializeOwned + From<EpochHandle>,
    C: Chromosome + Clone,
    T: Clone,
{
    fn read(&self, path: PathBuf) -> std::io::Result<Generation<C, T>> {
        Python::attach(|py| {
            let data = std::fs::read(path)?;
            let py_generation = match self.0.as_str() {
                JSON_EXTENSION => {
                    let json_str = String::from_utf8(data).map_err(|e| {
                        std::io::Error::other(format!(
                            "Failed to read checkpoint file as UTF-8 string: {}",
                            e
                        ))
                    })?;
                    PyGeneration::from_json(&json_str)?
                }
                PICKLE_EXTENSION => {
                    let pickle_bytes = PyBytes::new(py, &data);
                    PyGeneration::from_pickle(&pickle_bytes)?
                }
                _ => {
                    return Err(std::io::Error::other(format!(
                        "Unsupported checkpoint format: {}",
                        self.0
                    )));
                }
            };

            Ok(py_generation.inner.into())
        })
    }
}
