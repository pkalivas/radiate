use crate::ObjectValue;
use crate::PyMetricSet;
use crate::conversion::Wrap;
use pyo3::BoundObject;
use pyo3::IntoPyObject;
use pyo3::PyErr;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::Chromosome;
use radiate::Gene;
use radiate::Genotype;
use radiate::Metric;
use radiate::MetricSet;
use radiate::MultiObjectiveGeneration;
use radiate::{BitChromosome, CharChromosome, Epoch, FloatChromosome, Generation, IntChromosome};

use super::PyGenotype;

impl<'py, C> IntoPyObject<'py> for Wrap<Generation<C, ObjectValue>>
where
    C: Chromosome,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let generation = self.0;

        let result = PyDict::new(py);

        result.set_item("index", generation.index())?;

        let score = PyList::empty(py);
        for val in generation.score().values.iter() {
            score.append(*val).unwrap();
        }
        result.set_item("score", score)?;
        result.set_item("value", generation.value().clone().inner)?;
        result.set_item("metrics", Wrap(generation.metrics().clone()))?;
        panic!()
    }
}

impl<'py, C> IntoPyObject<'py> for Wrap<MultiObjectiveGeneration<C>>
where
    C: Chromosome + Clone,
    PyGenotype: From<Genotype<C>>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let generation = self.0;

        let result = PyDict::new(py);

        result.set_item("index", generation.index())?;
        result.set_item("value", py.None())?;
        result.set_item("metrics", Wrap(generation.metrics().clone()))?;

        let result = PyList::empty(py);
        for member in generation.value().values().iter() {
            let temp = PyGenotype::from(member.genotype().clone());
            // let genotype = member
            //     .genotype()
            //     .iter()
            //     .map(|chromosome| {
            //         let list = PyList::empty(py);
            //         for gene in chromosome.iter() {
            //             // let allele =
            //             // let any_value = Wrap(allele.into()).into_pyobject(py).unwrap();
            //             // let any_value = any_value_into_py_object(allele.into(), py)
            //             //     .expect("Failed to convert allele to AnyValue");
            //             // list.append(any_value).unwrap();
            //         }

            //         list.into_pyobject(py).unwrap()
            //     })
            //     .collect::<Vec<_>>();

            let fitness = member
                .score()
                .unwrap()
                .values
                .iter()
                .cloned()
                .collect::<Vec<_>>();

            let member = PyDict::new(py);
            member.set_item("genotype", temp)?;
            member.set_item("fitness", fitness)?;

            result.append(member)?;
        }

        // if let Some(pareto_front) = generation.pareto_front() {
        //     let pareto_list: Py<PyList> = Wrap(pareto_front).into_pyobject(py)?.unbind();
        //     result.set_item("pareto_front", pareto_list)?;
        // }

        Ok(result.into_any().into_bound())
    }
}

impl<'py> IntoPyObject<'py> for Wrap<MetricSet> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        metric_set_to_py_dict(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

impl<'py> IntoPyObject<'py> for Wrap<Metric> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        metric_to_py_dict(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

fn metric_set_to_py_dict<'py, 'a>(
    py: Python<'py>,
    metric_set: &MetricSet,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    for (name, metric) in metric_set.iter() {
        let metric_dict = metric_to_py_dict(py, metric)?;
        dict.set_item(name, metric_dict)?;
    }

    Ok(dict)
}

fn metric_to_py_dict<'py, 'a>(py: Python<'py>, metric: &Metric) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("value_last", metric.last_value())?;
    dict.set_item("value_mean", metric.value_mean())?;
    dict.set_item("value_stddev", metric.value_std_dev())?;
    dict.set_item("value_variance", metric.value_variance())?;
    dict.set_item("value_skewness", metric.value_skewness())?;
    dict.set_item("value_min", metric.value_min())?;
    dict.set_item("value_max", metric.value_max())?;
    dict.set_item("value_count", metric.count())?;

    dict.set_item("sequence_last", metric.last_sequence())?;
    dict.set_item("sequence_mean", metric.distribution_mean())?;
    dict.set_item("sequence_stddev", metric.distribution_std_dev())?;
    dict.set_item("sequence_min", metric.distribution_min())?;
    dict.set_item("sequence_max", metric.distribution_max())?;
    dict.set_item("sequence_variance", metric.distribution_variance())?;
    dict.set_item("sequence_skewness", metric.distribution_skewness())?;
    dict.set_item("sequence_kurtosis", metric.distribution_kurtosis())?;

    dict.set_item("time_last", metric.last_time())?;
    dict.set_item("time_sum", metric.time_sum())?;
    dict.set_item("time_mean", metric.time_mean())?;
    dict.set_item("time_std_dev", metric.time_std_dev())?;
    dict.set_item("time_min", metric.time_min())?;
    dict.set_item("time_max", metric.time_max())?;
    dict.set_item("time_variance", metric.time_variance())?;

    let result = PyDict::new(py);
    result.set_item("name", metric.name())?;
    result.set_item("metrics", dict)?;

    Ok(result)
}

#[pyclass(unsendable)]
pub struct PyGeneration {
    pub index: usize,
    pub score: Py<PyList>,
    pub value: Py<PyAny>,
    pub metrics: PyMetricSet,
    pub pareto_front: Option<Py<PyList>>,
}

#[pymethods]
impl PyGeneration {
    #[new]
    #[pyo3(signature = (index, score, value, metrics, pareto_front=None))]
    pub fn new(
        index: usize,
        score: Py<PyList>,
        value: Py<PyAny>,
        metrics: PyMetricSet,
        pareto_front: Option<Py<PyList>>,
    ) -> Self {
        Self {
            index,
            score,
            value,
            metrics,
            pareto_front,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.score.as_any().into_bound_py_any(py)
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.value.as_any().into_bound_py_any(py)
    }

    pub fn get_metric<'py>(
        &self,
        py: Python<'py>,
        name: String,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        self.metrics
            .get_metric(name)
            .map(|metric| metric.into_bound_py_any(py))
            .transpose()
    }

    pub fn get_pareto_front<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if let Some(pareto_front) = &self.pareto_front {
            let temp = pareto_front.as_any().bind(py).to_owned();
            Ok(Some(temp))
        } else {
            Ok(None)
        }
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;

        Ok(format!(
            "Generation(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
            self.index,
            score,
            value,
            self.metrics.__repr__()
        ))
    }
}

impl Into<PyGeneration> for Generation<FloatChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}

impl Into<PyGeneration> for Generation<IntChromosome<i32>, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}
impl Into<PyGeneration> for Generation<CharChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}
impl Into<PyGeneration> for Generation<BitChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}

// impl Into<PyGeneration> for MultiObjectiveGeneration<IntChromosome<i32>> {
//     fn into(self) -> PyGeneration {
//         Python::with_gil(|py| {
//             let score = PyList::empty(py);
//             let pareto_front = self
//                 .value()
//                 .values()
//                 .iter()
//                 .map(|phenotype| (*(*phenotype)).clone())
//                 .collect::<Vec<_>>();

//             PyGeneration {
//                 index: self.index(),
//                 score: score.unbind(),
//                 value: py.None(),
//                 metrics: self.metrics().clone().into(),
//                 pareto_front: Some(Wrap(pareto_front).into_pyobject(py).unwrap().unbind()),
//             }
//         })
//     }
// }
// impl Into<PyGeneration> for MultiObjectiveGeneration<FloatChromosome> {
//     fn into(self) -> PyGeneration {
//         Python::with_gil(|py| {
//             let score = PyList::empty(py);
//             let pareto_front = self
//                 .value()
//                 .values()
//                 .iter()
//                 .map(|phenotype| (*(*phenotype)).clone())
//                 .collect::<Vec<_>>();

//             PyGeneration {
//                 index: self.index(),
//                 score: score.unbind(),
//                 value: py.None(),
//                 metrics: self.metrics().clone().into(),
//                 pareto_front: Some(Wrap(pareto_front).into_pyobject(py).unwrap().unbind()),
//             }
//         })
//     }
// }
