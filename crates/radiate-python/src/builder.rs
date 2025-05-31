use crate::{ObjectValue, PyGeneType};
use pyo3::{
    Borrowed, Bound, FromPyObject, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyResult,
    Python,
    conversion::FromPyObjectBound,
    pyclass, pymethods,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods, PyString},
};
use std::{borrow::Borrow, vec};

#[pyclass]
#[derive(Clone, Debug)]
pub enum OperatorType {
    Selection,
    Crossover,
    Mutation,
    Replacement,
}

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub struct Operator {
    name: String,
    op_type: OperatorType,
    args: ObjectValue,
    allowed_genes: Vec<PyGeneType>,
}

#[pymethods]
impl Operator {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn op_type<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self.op_type {
            OperatorType::Selection => PyString::new(py, "selection").into_bound_py_any(py),
            OperatorType::Crossover => PyString::new(py, "crossover").into_bound_py_any(py),
            OperatorType::Mutation => PyString::new(py, "mutation").into_bound_py_any(py),
            OperatorType::Replacement => PyString::new(py, "replacement").into_bound_py_any(py),
        }
    }

    pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.args.inner.bind(py).into_bound_py_any(py)
    }

    pub fn allowed_genes(&self) -> PyResult<Vec<PyGeneType>> {
        Ok(self.allowed_genes.clone())
    }

    pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.__repr__(py)
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!(
            "Operator(name={}, type={:?}, args={})",
            self.name,
            self.op_type,
            self.args.inner.bind(py)
        );

        PyString::new(py, &repr).into_bound_py_any(py)
    }

    ///
    /// Crossovers
    ///

    #[staticmethod]
    #[pyo3(signature = (rate=None, alpha=None))]
    pub fn blend_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        alpha: Option<f32>,
    ) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(a) = alpha {
            args.set_item("alpha", a)?;
        }

        Ok(Operator {
            name: "blend_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None, alpha=None))]
    pub fn intermediate_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        alpha: Option<f32>,
    ) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(a) = alpha {
            args.set_item("alpha", a)?;
        }

        Ok(Operator {
            name: "intermediate_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn mean_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "mean_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float, PyGeneType::Int],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn multi_point_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "multi_point_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn shuffle_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "shuffle_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None, contiguty=None))]
    pub fn simulated_binary_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        contiguty: Option<f32>,
    ) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(c) = contiguty {
            args.set_item("contiguty", c)?;
        }

        Ok(Operator {
            name: "simulated_binary_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn uniform_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "uniform_crossover".into(),
            op_type: OperatorType::Crossover,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    ///
    /// Mutators
    ///

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn arithmetic_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "arithmetic_mutator".into(),
            op_type: OperatorType::Mutation,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn gaussian_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "gaussian_mutator".into(),
            op_type: OperatorType::Mutation,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn scramble_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "scramble_mutator".into(),
            op_type: OperatorType::Mutation,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn swap_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "swap_mutator".into(),
            op_type: OperatorType::Mutation,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn uniform_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(Operator {
            name: "uniform_mutator".into(),
            op_type: OperatorType::Mutation,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    ///
    /// Selectors
    ///

    #[staticmethod]
    #[pyo3(signature = (tournament_size=None))]
    pub fn tournament_selector<'py>(
        py: Python<'py>,
        tournament_size: Option<usize>,
    ) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(size) = tournament_size {
            args.set_item("tournament_size", size)?;
        }

        Ok(Operator {
            name: "tournament_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn roulette_wheel_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "roulette_wheel_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn rank_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "rank_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn steady_state_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "steady_state_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn stochastic_universal_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "stochastic_universal_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (temp=None))]
    pub fn boltzmann_selector<'py>(py: Python<'py>, temp: Option<f32>) -> PyResult<Operator> {
        let args = PyDict::new(py);
        if let Some(t) = temp {
            args.set_item("temp", t)?;
        }

        Ok(Operator {
            name: "boltzmann_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn elite_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "elite_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn random_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "random_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn nsga2_selector<'py>(py: Python<'py>) -> PyResult<Operator> {
        let args = PyDict::new(py);

        Ok(Operator {
            name: "nsga2_selector".into(),
            op_type: OperatorType::Selection,
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
        })
    }
}

// Removed manual FromPyObjectBound implementation for Operator as it is a sealed trait.
// If you need to extract Operator from Python, implement FromPyObject for Operator or use a conversion function.

#[pyclass]
pub struct EngineBuilderTemp {
    population_size: usize,
    offspring_fraction: f32,
    operators: Vec<Operator>,
}

#[pymethods]
impl EngineBuilderTemp {
    #[new]
    #[pyo3(signature = (population_size=100, offspring_fraction=0.5, operators=None))]
    pub fn new<'py>(
        py: Python<'py>,
        population_size: usize,
        offspring_fraction: f32,
        operators: Option<Bound<'py, PyList>>,
    ) -> PyResult<Self> {
        let operators_vec = if let Some(obj) = operators {
            if obj.is_none() {
                Vec::new()
            } else {
                let t = PyList::new(py, obj)?;
                t.try_iter()
                    .map(|op| op.extract::<Operator>())
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Operator>>()
            }
        } else {
            Vec::new()
        };
        Ok(Self {
            population_size,
            offspring_fraction,
            operators: operators_vec,
        })
    }

    pub fn population_size(&self) -> usize {
        self.population_size
    }

    pub fn offspring_fraction(&self) -> f32 {
        self.offspring_fraction
    }
    pub fn operators<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if self.operators.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No operators defined",
            ));
        }

        let list = PyList::empty(py);
        for op in &self.operators {
            let op_bound = op.clone().into_bound_py_any(py)?;
            list.append(op_bound)?;
        }
        Ok(ObjectValue { inner: list.into() }.into_bound_py_any(py)?)
    }
}
