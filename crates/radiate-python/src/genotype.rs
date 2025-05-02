use pyo3::{Py, PyObject, PyResult, Python, pyclass, pymethods};

// #[pyclass(name = "AnyGenotype", sequence)]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyAnyGenotype {
//     pub inner: Vec<PyAnyChromosome>,
// }

// #[pymethods]
// impl PyAnyGenotype {
//     #[new]
//     #[pyo3(signature = (chromosomes))]
//     pub fn new(py: Python, chromosomes: PyObject) -> PyResult<Self> {
//         let chromosomes: Vec<PyAnyChromosome> = chromosomes.extract(py)?;
//         Ok(PyAnyGenotype { inner: chromosomes })
//     }

//     pub fn __getitem__(&self, index: usize) -> PyResult<PyAnyGenotype> {
//         let inner = self.inner.clone();
//         Ok(PyAnyGenotype { inner })
//     }

//     pub fn __len__(&self) -> usize {
//         self.inner.len()
//     }
// }

// #[derive(Clone, Debug)]
// pub enum InnerGenotype {
//     Float(Vec<FloatChromosome>),
//     Int(Vec<IntChromosome<i32>>),
//     Bit(Vec<BitChromosome>),
//     Char(Vec<CharChromosome>),
//     Any(Vec<AnyChromosome<'static>>),
// }

// #[pyclass(name = "Genotype", sequence)]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyGenotype {
//     pub inner: InnerGenotype,
// }
