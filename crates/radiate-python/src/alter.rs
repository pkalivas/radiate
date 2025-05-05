use std::collections::BTreeMap;

use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyEngineParam {
    pub name: String,
    pub args: BTreeMap<String, String>,
}

#[pymethods]
impl PyEngineParam {
    #[new]
    #[pyo3(signature = (name, args=None))]
    pub fn new(name: String, args: Option<BTreeMap<String, String>>) -> Self {
        Self {
            name,
            args: args.unwrap_or_default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_args(&self) -> &BTreeMap<String, String> {
        &self.args
    }

    pub fn get_arg(&self, key: &str) -> Option<&String> {
        self.args.get(key)
    }
}
