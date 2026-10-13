use crate::{PyAnyObject, bindings::converters::FromPyEngineInput};
use pyo3::{Py, PyAny, Python, prelude::FromPyObjectOwned, pyclass, pymethods};
use radiate::RadiateResult;
use radiate_error::{radiate_bail, radiate_err};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum PyEngineInputType {
    Alterer,
    OffspringSelector,
    SurvivorSelector,
    Diversity,
    Objective,
    Limit,
    PopulationSize,
    OffspringFraction,
    MaxSpeciesAge,
    MaxPhenotypeAge,
    FrontRange,
    Executor,
    Evaluator,
    SpeciesThreshold,
    TargetSpecies,
    Population,
    Subscriber,
    Generation,
    CheckpointLoad,
    CheckpointWrite,
    Metric,
    Codec,
    FitnessFunction,
    Filter,
    Unknown,
}

#[pyclass(from_py_object)]
#[derive(Clone, Serialize, Deserialize)]
pub struct PyEngineInput {
    pub component: Option<String>,
    pub input_type: PyEngineInputType,
    pub args: HashMap<String, PyAnyObject>,
}

#[pymethods]
impl PyEngineInput {
    #[new]
    pub fn new(
        input_type: PyEngineInputType,
        component: Option<String>,
        args: HashMap<String, Py<PyAny>>,
    ) -> Self {
        PyEngineInput {
            component,
            input_type,
            args: args
                .into_iter()
                .map(|(k, v)| (k, PyAnyObject { inner: v }))
                .collect(),
        }
    }

    pub fn component(&self) -> &str {
        self.component.as_deref().unwrap_or("")
    }

    pub fn input_type(&self) -> PyEngineInputType {
        self.input_type
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PyEngineInput {
    pub fn expect_type(&self, expected: PyEngineInputType) -> RadiateResult<()> {
        if self.input_type != expected {
            radiate_bail!(Builder: "Expected input type {:?}, but found {:?}", expected, self.input_type);
        }

        Ok(())
    }

    pub fn convert<T: FromPyEngineInput>(&self) -> RadiateResult<T> {
        T::from_input(self)
    }

    pub fn extract<T: for<'py> FromPyObjectOwned<'py>>(&self, key: &str) -> RadiateResult<T> {
        let Some(value) = self.args.get(key) else {
            radiate_bail!(Builder: "Key '{}' not found in PyEngineInput args", key);
        };

        Python::attach(|py| value.extract::<T>(py)).map_err(|e| {
            radiate_err!(Builder: "Failed to extract key '{}' from PyEngineInput args: {}", key, e)
        })
    }

    pub fn get(&self, key: &str) -> Option<&PyAnyObject> {
        self.args.get(key)
    }
}

impl Debug for PyEngineInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = self
            .args
            .iter()
            .map(|(k, v)| format!("\t\t{}: {:?}", k, v))
            .collect::<Vec<String>>()
            .join("\n");
        if !args.is_empty() {
            args = format!("\n{}", args);
            args.push('\n');
        }
        write!(
            f,
            "PyEngineInput {{ \n\tcomponent: {:?}, \n\tinput_type: {:?}, \n\targs: {{{}}} \n}}",
            self.component, self.input_type, args
        )
    }
}
