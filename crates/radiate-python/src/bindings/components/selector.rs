use crate::{ObjectValue, PyGeneType, conversion::Wrap, gene::PyChromosomeType};
use pyo3::{
    Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python, pyclass, pymethods,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyString},
};
use radiate::{
    BoltzmannSelector, Chromosome, EliteSelector, NSGA2Selector, RandomSelector, RankSelector,
    RouletteSelector, Select, StochasticUniversalSamplingSelector, TournamentSelector,
};
use std::{hash::Hash, vec};

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub struct PySelector {
    name: String,
    args: ObjectValue,
    allowed_genes: Vec<PyGeneType>,
    chromosome_types: Vec<PyChromosomeType>,
}

#[pymethods]
impl PySelector {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.__repr__(py)
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!(
            "Selector(name={}, args={})",
            self.name,
            self.args.inner.bind(py)
        );

        PyString::new(py, &repr).into_bound_py_any(py)
    }

    pub fn __eq__<'py>(&self, other: &Self) -> bool {
        let mut state = std::hash::DefaultHasher::new();
        self.name == other.name && self.args.hash(&mut state) == other.args.hash(&mut state)
    }

    pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.args.inner.bind(py).into_bound_py_any(py)
    }

    pub fn allowed_genes(&self) -> PyResult<Vec<PyGeneType>> {
        Ok(self.allowed_genes.clone())
    }

    pub fn is_valid_for_chromosome(&self, chromosome_type: &str) -> bool {
        self.chromosome_types
            .iter()
            .any(|c| c.name() == chromosome_type)
    }

    #[staticmethod]
    #[pyo3(signature = (tournament_size=None))]
    pub fn tournament_selector<'py>(
        py: Python<'py>,
        tournament_size: Option<usize>,
    ) -> PyResult<PySelector> {
        let args = PyDict::new(py);
        if let Some(size) = tournament_size {
            args.set_item("tournament_size", size)?;
        }

        Ok(PySelector {
            name: "tournament_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn roulette_wheel_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "roulette_wheel_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn rank_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "rank_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn steady_state_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "steady_state_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn stochastic_universal_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "stochastic_universal_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (temp=None))]
    pub fn boltzmann_selector<'py>(py: Python<'py>, temp: Option<f32>) -> PyResult<PySelector> {
        let args = PyDict::new(py);
        if let Some(t) = temp {
            args.set_item("temp", t)?;
        }

        Ok(PySelector {
            name: "boltzmann_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn elite_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "elite_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn random_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "random_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn nsga2_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "nsga2_selector".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosome_types: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }
}

impl<'py> FromPyObject<'py> for Wrap<TournamentSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "tournament_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a tournament_selector",
            ));
        }

        let args = selector.args.inner.bind(ob.py());
        let tournament_size = args.get_item("tournament_size").and_then(|v| v.extract())?;

        Ok(Wrap(TournamentSelector::new(tournament_size)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<RouletteSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "roulette_wheel_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a roulette_wheel_selector",
            ));
        }

        Ok(Wrap(RouletteSelector::default()))
    }
}

// Rank selector
impl<'py> FromPyObject<'py> for Wrap<RankSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "rank_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a rank_selector",
            ));
        }

        Ok(Wrap(RankSelector::default()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<StochasticUniversalSamplingSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "stochastic_universal_sampling_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a stochastic_universal_sampling_selector",
            ));
        }

        Ok(Wrap(StochasticUniversalSamplingSelector::new()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<BoltzmannSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "boltzmann_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a boltzmann_selector",
            ));
        }

        let args = selector.args.inner.bind(ob.py());
        let temp = args.get_item("temp").and_then(|v| v.extract())?;

        Ok(Wrap(BoltzmannSelector::new(temp)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<EliteSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "elite_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a elite_selector",
            ));
        }

        Ok(Wrap(EliteSelector::default()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<RandomSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "random_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a random_selector",
            ));
        }

        Ok(Wrap(RandomSelector::default()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<NSGA2Selector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != "nsga2_selector" {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a nsga2_selector",
            ));
        }

        Ok(Wrap(NSGA2Selector::new()))
    }
}

impl<'py, C> FromPyObject<'py> for Wrap<Box<dyn Select<C>>>
where
    C: Chromosome + Clone + 'static,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        let py_selector: Box<dyn Select<C>> = match selector.name.as_str() {
            "tournament_selector" => Box::new(ob.extract::<Wrap<TournamentSelector>>()?.0),
            "roulette_wheel_selector" => Box::new(ob.extract::<Wrap<RouletteSelector>>()?.0),
            "rank_selector" => Box::new(ob.extract::<Wrap<RankSelector>>()?.0),
            "stochastic_universal_sampling_selector" => {
                Box::new(ob.extract::<Wrap<StochasticUniversalSamplingSelector>>()?.0)
            }
            "boltzmann_selector" => Box::new(ob.extract::<Wrap<BoltzmannSelector>>()?.0),
            "elite_selector" => Box::new(ob.extract::<Wrap<EliteSelector>>()?.0),
            "random_selector" => Box::new(ob.extract::<Wrap<RandomSelector>>()?.0),
            "nsga2_selector" => Box::new(ob.extract::<Wrap<NSGA2Selector>>()?.0),
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Unknown selector type",
                ));
            }
        };

        Ok(Wrap(py_selector))
    }
}
