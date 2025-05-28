mod float;
mod int;

pub use float::{PyFloatChromosome, PyFloatGene};
pub use int::{PyIntChromosome, PyIntGene};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyObject, PyResult, Python, pyclass, pymethods};

#[pyclass]
pub enum PyGene {
    Float(PyFloatGene),
    Int(PyIntGene),
}

#[pymethods]
impl PyGene {
    #[new]
    pub fn new(py: Python<'_>, gene: PyObject) -> Self {
        if let Ok(float_gene) = gene.extract::<PyFloatGene>(py) {
            PyGene::Float(float_gene)
        } else if let Ok(int_gene) = gene.extract::<PyIntGene>(py) {
            PyGene::Int(int_gene)
        } else {
            panic!("Unsupported gene type")
        }
    }

    pub fn allele<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self {
            PyGene::Float(gene) => gene.allele().into_bound_py_any(py),
            PyGene::Int(gene) => gene.allele().into_bound_py_any(py),
        }
    }
}

#[pyclass]
pub struct PyChromosome {
    pub genes: Vec<PyGene>,
}

// use crate::conversion::{Wrap, any_value_into_py_object};
// use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods};
// use radiate::{AnyGene, BitGene, CharGene, FloatGene, Gene, IntGene, object::AnyValue};

// #[pyclass(name = "FloatGene")]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyFloatGene {
//     pub inner: FloatGene,
// }

// #[pymethods]
// impl PyFloatGene {
//     #[new]
//     #[pyo3(signature=(min=0.0, max=1.0, min_bound=None, max_bound=None))]
//     pub fn new(min: f32, max: f32, min_bound: Option<f32>, max_bound: Option<f32>) -> Self {
//         let range = min..max;
//         let bounds = if let (Some(min_bound), Some(max_bound)) = (min_bound, max_bound) {
//             min_bound..max_bound
//         } else {
//             range.clone()
//         };

//         Self {
//             inner: FloatGene::from((range, bounds)),
//         }
//     }

//     pub fn allele(&self) -> &f32 {
//         self.inner.allele()
//     }
// }

// impl AsRef<FloatGene> for PyFloatGene {
//     fn as_ref(&self) -> &FloatGene {
//         &self.inner
//     }
// }

// #[pyclass(name = "IntGene")]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyIntGene {
//     pub inner: IntGene<i32>,
// }

// #[pymethods]
// impl PyIntGene {
//     #[new]
//     #[pyo3(signature=(min=0, max=100, min_bound=None, max_bound=None))]
//     pub fn new(min: i32, max: i32, min_bound: Option<i32>, max_bound: Option<i32>) -> Self {
//         let range = min..max;
//         let bounds = if let (Some(min_bound), Some(max_bound)) = (min_bound, max_bound) {
//             min_bound..max_bound
//         } else {
//             range.clone()
//         };

//         Self {
//             inner: IntGene::from((range, bounds)),
//         }
//     }

//     pub fn allele(&self) -> &i32 {
//         self.inner.allele()
//     }
// }

// #[pyclass(name = "CharGene")]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyCharGene {
//     pub inner: CharGene,
// }

// #[pymethods]
// impl PyCharGene {
//     #[new]
//     #[pyo3(signature=(allele=None, char_set=None))]
//     pub fn new(allele: Option<char>, char_set: Option<String>) -> Self {
//         let inner = if let Some(allele) = allele {
//             if let Some(char_set) = char_set {
//                 CharGene::from((allele, char_set.chars().collect::<Vec<char>>().into()))
//             } else {
//                 CharGene::from(allele)
//             }
//         } else {
//             CharGene::default()
//         };

//         Self { inner }
//     }

//     pub fn allele(&self) -> &char {
//         self.inner.allele()
//     }
// }

// #[pyclass(name = "BitGene")]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyBitGene {
//     pub inner: BitGene,
// }

// #[pymethods]
// impl PyBitGene {
//     #[new]
//     pub fn new() -> Self {
//         Self {
//             inner: BitGene::new(),
//         }
//     }

//     pub fn allele(&self) -> &bool {
//         self.inner.allele()
//     }
// }

// #[pyclass(name = "AnyGene")]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyAnyGene {
//     pub inner: AnyGene<'static>,
// }

// #[pymethods]
// impl PyAnyGene {
//     #[new]
//     pub fn new(allele: Option<Wrap<AnyValue<'_>>>) -> Self {
//         let inner = if let Some(allele) = allele.as_ref() {
//             AnyGene::new(allele.0.clone().into_static())
//         } else {
//             AnyGene::new(AnyValue::Null)
//         };

//         Self { inner }
//     }

//     pub fn allele<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         any_value_into_py_object(self.inner.allele().clone(), py)
//     }
// }

// impl AsRef<AnyGene<'static>> for PyAnyGene {
//     fn as_ref(&self) -> &AnyGene<'static> {
//         &self.inner
//     }
// }
