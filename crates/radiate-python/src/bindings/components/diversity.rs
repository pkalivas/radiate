use crate::{
    ObjectValue,
    bindings::genome::{PyChromosomeType, PyGeneType},
    conversion::Wrap,
};
use pyo3::{
    Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python, pyclass, pymethods,
    types::{PyAnyMethods, PyDict, PyString},
};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, Diversity, EuclideanDistance, FloatChromosome, Gene,
    GraphChromosome, HammingDistance, IntChromosome, NeatDistance, Op,
};
use std::hash::Hash;

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyDiversity {
    name: String,
    args: ObjectValue,
    allowed_genes: Vec<PyGeneType>,
    chromosomes: Vec<PyChromosomeType>,
}

#[pymethods]
impl PyDiversity {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.__repr__(py)
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!(
            "Alterer(name={}, args={})",
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
        self.chromosomes.iter().any(|c| c.name() == chromosome_type)
    }

    #[staticmethod]
    pub fn hamming_distance<'py>(py: Python<'py>) -> PyResult<PyDiversity> {
        Ok(PyDiversity {
            name: "HammingDistance".to_string(),
            args: ObjectValue { inner: py.None() },
            allowed_genes: vec![
                PyGeneType::Bit,
                PyGeneType::Int,
                PyGeneType::Float,
                PyGeneType::Char,
            ],
            chromosomes: vec![
                PyChromosomeType::Bit,
                PyChromosomeType::Int,
                PyChromosomeType::Float,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    pub fn euclidean_distance<'py>(py: Python<'py>) -> PyResult<PyDiversity> {
        Ok(PyDiversity {
            name: "EuclideanDistance".to_string(),
            args: ObjectValue { inner: py.None() },
            allowed_genes: vec![PyGeneType::Float],
            chromosomes: vec![PyChromosomeType::Float],
        })
    }

    #[staticmethod]
    pub fn neat_distance<'py>(
        py: Python<'py>,
        excess: f32,
        disjoint: f32,
        weight_diff: f32,
    ) -> PyResult<PyDiversity> {
        let args = PyDict::new(py);
        args.set_item("excess", excess).unwrap();
        args.set_item("disjoint", disjoint).unwrap();
        args.set_item("weight_diff", weight_diff).unwrap();
        Ok(PyDiversity {
            name: "NeatDistance".to_string(),
            args: ObjectValue {
                inner: args.unbind().into_any(),
            },
            allowed_genes: vec![PyGeneType::Graph],
            chromosomes: vec![PyChromosomeType::Graph],
        })
    }
}

impl<'py, C, G> FromPyObject<'py> for Wrap<Option<Box<dyn Diversity<C>>>>
where
    C: Chromosome<Gene = G>,
    G: Gene,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let diversity = if let Ok(div) = ob.extract::<PyDiversity>() {
            div
        } else {
            return Ok(Wrap(None));
        };

        let chromosome_name = std::any::type_name::<C>()
            .split("::")
            .filter(|s| s.contains("Chromosome"))
            .map(|s| s.split('<').next().unwrap_or_default())
            .next()
            .unwrap_or_default();

        if !diversity.is_valid_for_chromosome(chromosome_name) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Diversity '{}' is not valid for chromosome '{}'",
                diversity.name, chromosome_name
            )));
        }

        match chromosome_name {
            "BitChromosome" => {
                let div = if diversity.name == "HammingDistance" {
                    Ok(Box::new(HammingDistance))
                } else {
                    Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "HammingDistance is not supported for BitChromosome",
                    ))
                };

                if let Ok(div) = div {
                    return Ok(Wrap(Some(unsafe {
                        std::mem::transmute::<
                            Box<dyn Diversity<BitChromosome>>,
                            Box<dyn Diversity<C>>,
                        >(div)
                    })));
                }
            }
            "FloatChromosome" => {
                let div = if diversity.name == "EuclideanDistance" {
                    Ok(Box::new(EuclideanDistance) as Box<dyn Diversity<FloatChromosome>>)
                } else if diversity.name == "HammingDistance" {
                    Ok(Box::new(HammingDistance) as Box<dyn Diversity<FloatChromosome>>)
                } else {
                    Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "EuclideanDistance is not supported for FloatChromosome",
                    ))
                };

                if let Ok(div) = div {
                    return Ok(Wrap(Some(unsafe {
                        std::mem::transmute::<
                            Box<dyn Diversity<radiate::FloatChromosome>>,
                            Box<dyn Diversity<C>>,
                        >(div)
                    })));
                }
            }
            "IntChromosome" => {
                let div = if diversity.name == "HammingDistance" {
                    Ok(Box::new(HammingDistance) as Box<dyn Diversity<IntChromosome<i32>>>)
                } else {
                    Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "HammingDistance is not supported for IntChromosome",
                    ))
                };

                if let Ok(div) = div {
                    return Ok(Wrap(Some(unsafe {
                        std::mem::transmute::<
                            Box<dyn Diversity<IntChromosome<i32>>>,
                            Box<dyn Diversity<C>>,
                        >(div)
                    })));
                }
            }
            "CharChromosome" => match diversity.name.as_str() {
                "HammingDistance" => {
                    let div = Box::new(HammingDistance) as Box<dyn Diversity<CharChromosome>>;

                    return Ok(Wrap(Some(unsafe {
                        std::mem::transmute::<
                            Box<dyn Diversity<CharChromosome>>,
                            Box<dyn Diversity<C>>,
                        >(div)
                    })));
                }
                _ => {}
            },
            "GraphChromosome" => match diversity.name.as_str() {
                "NeatDistance" => {
                    let args = diversity.args.inner.bind(ob.py());
                    let excess: f32 = args.get_item("excess")?.extract()?;
                    let disjoint: f32 = args.get_item("disjoint")?.extract()?;
                    let weight_diff: f32 = args.get_item("weight_diff")?.extract()?;
                    let div = Box::new(NeatDistance::new(excess, disjoint, weight_diff))
                        as Box<dyn Diversity<GraphChromosome<Op<f32>>>>;
                    return Ok(Wrap(Some(unsafe {
                        std::mem::transmute::<
                            Box<dyn Diversity<GraphChromosome<Op<f32>>>>,
                            Box<dyn Diversity<C>>,
                        >(div)
                    })));
                }
                _ => {}
            },
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Unsupported chromosome type",
                ));
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Unsupported diversity type",
        ))
    }
}

impl<'py> FromPyObject<'py> for Wrap<HammingDistance> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let args = ob.extract::<PyDiversity>()?;

        if args.name != "HammingDistance" {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Expected HammingDistance diversity",
            ));
        }

        Ok(Wrap(HammingDistance))
    }
}

impl<'py> FromPyObject<'py> for Wrap<EuclideanDistance> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let args = ob.extract::<PyDiversity>()?;

        if args.allowed_genes != vec![PyGeneType::Float] {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "EuclideanDistance only supports Float genes",
            ));
        }

        if args.name != "EuclideanDistance" {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Expected EuclideanDistance diversity",
            ));
        }

        Ok(Wrap(EuclideanDistance))
    }
}

impl<'py> FromPyObject<'py> for Wrap<NeatDistance> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let args = ob.extract::<PyDiversity>()?;

        if args.allowed_genes != vec![PyGeneType::Graph] {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "NeatDistance only supports Graph genes",
            ));
        }

        if args.name != "NeatDistance" {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Expected NeatDistance diversity",
            ));
        }

        let args = args.args.inner.bind(ob.py());
        let excess: f32 = args.get_item("excess")?.extract()?;
        let disjoint: f32 = args.get_item("disjoint")?.extract()?;
        let weight_diff: f32 = args.get_item("weight_diff")?.extract()?;

        Ok(Wrap(NeatDistance::new(excess, disjoint, weight_diff)))
    }
}
