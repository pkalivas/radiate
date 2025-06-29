use crate::{ObjectValue, PyChromosomeType, PyEngineInput, PyGeneType, conversion::Wrap};
use pyo3::{
    Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python, pyclass, pymethods,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyString},
};
use radiate::{
    BoltzmannSelector, Chromosome, EliteSelector, NSGA2Selector, RandomSelector, RankSelector,
    RouletteSelector, Select, SteadyStateSelector, StochasticUniversalSamplingSelector,
    TournamentNSGA2Selector, TournamentSelector,
};
use std::{hash::Hash, vec};

const TOURNAMENT_SELECTOR: &str = "TournamentSelector";
const ROULETTE_WHEEL_SELECTOR: &str = "RouletteWheelSelector";
const RANK_SELECTOR: &str = "RankSelector";
const STEADY_STATE_SELECTOR: &str = "SteadyStateSelector";
const STOCHASTIC_UNIVERSAL_SELECTOR: &str = "StochasticUniversalSamplingSelector";
const BOLTZMANN_SELECTOR: &str = "BoltzmannSelector";
const ELITE_SELECTOR: &str = "EliteSelector";
const RANDOM_SELECTOR: &str = "RandomSelector";
const NSGA2_SELECTOR: &str = "NSGA2Selector";
const TOURNAMENT_NSGA2_SELECTOR: &str = "TournamentNSGA2Selector";

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
    pub fn steady_state_selector<'py>(
        py: Python<'py>,
        replacement_count: Option<usize>,
    ) -> PyResult<PySelector> {
        let args = PyDict::new(py);
        if let Some(count) = replacement_count {
            args.set_item("replacement_count", count)?;
        }

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

    #[staticmethod]
    pub fn tournamen_nsga2_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
        let args = PyDict::new(py);

        Ok(PySelector {
            name: "tournament_nsga2_selector".into(),
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

impl<'py> FromPyObject<'py> for Wrap<SteadyStateSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != STEADY_STATE_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a steady_state_selector",
            ));
        }

        let replacement_count = selector
            .args
            .get("replacement_count")
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(10);

        Ok(Wrap(SteadyStateSelector::new(replacement_count)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<TournamentSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != TOURNAMENT_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a tournament_selector",
            ));
        }

        let tournament_size = selector
            .args
            .get("tournament_size")
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        Ok(Wrap(TournamentSelector::new(tournament_size)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<RouletteSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PySelector = ob.extract()?;
        if selector.name != ROULETTE_WHEEL_SELECTOR {
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
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != RANK_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a rank_selector",
            ));
        }

        Ok(Wrap(RankSelector::default()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<StochasticUniversalSamplingSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != STOCHASTIC_UNIVERSAL_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a stochastic_universal_sampling_selector",
            ));
        }

        Ok(Wrap(StochasticUniversalSamplingSelector::new()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<BoltzmannSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != BOLTZMANN_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a boltzmann_selector",
            ));
        }

        let temp = selector
            .args
            .get("temp")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);

        Ok(Wrap(BoltzmannSelector::new(temp)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<EliteSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != ELITE_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a elite_selector",
            ));
        }

        Ok(Wrap(EliteSelector::default()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<RandomSelector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != RANDOM_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a random_selector",
            ));
        }

        Ok(Wrap(RandomSelector::default()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<NSGA2Selector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != NSGA2_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a nsga2_selector",
            ));
        }

        Ok(Wrap(NSGA2Selector::new()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<TournamentNSGA2Selector> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        if selector.component != TOURNAMENT_NSGA2_SELECTOR {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a tournament_nsga2_selector",
            ));
        }

        Ok(Wrap(TournamentNSGA2Selector::new()))
    }
}

impl<'py, C> FromPyObject<'py> for Wrap<Box<dyn Select<C>>>
where
    C: Chromosome + Clone + 'static,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let selector: PyEngineInput = ob.extract()?;
        let py_selector: Box<dyn Select<C>> = match selector.component.as_str() {
            TOURNAMENT_SELECTOR => Box::new(ob.extract::<Wrap<TournamentSelector>>()?.0),
            ROULETTE_WHEEL_SELECTOR => Box::new(ob.extract::<Wrap<RouletteSelector>>()?.0),
            RANK_SELECTOR => Box::new(ob.extract::<Wrap<RankSelector>>()?.0),
            STOCHASTIC_UNIVERSAL_SELECTOR => {
                Box::new(ob.extract::<Wrap<StochasticUniversalSamplingSelector>>()?.0)
            }
            BOLTZMANN_SELECTOR => Box::new(ob.extract::<Wrap<BoltzmannSelector>>()?.0),
            ELITE_SELECTOR => Box::new(ob.extract::<Wrap<EliteSelector>>()?.0),
            RANDOM_SELECTOR => Box::new(ob.extract::<Wrap<RandomSelector>>()?.0),
            NSGA2_SELECTOR => Box::new(ob.extract::<Wrap<NSGA2Selector>>()?.0),
            TOURNAMENT_NSGA2_SELECTOR => Box::new(ob.extract::<Wrap<TournamentNSGA2Selector>>()?.0),
            STEADY_STATE_SELECTOR => Box::new(ob.extract::<Wrap<SteadyStateSelector>>()?.0),
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Unknown selector type",
                ));
            }
        };

        Ok(Wrap(py_selector))
    }
}
