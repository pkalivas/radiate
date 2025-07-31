use pyo3::{
    Bound, Py, PyAny, PyResult, Python, exceptions::PyTypeError, pyclass, pymethods,
    types::PyAnyMethods,
};
use radiate::{
    CosineDistance, EuclideanDistance, FitnessFunction, Graph, GraphArchitectureNovelty,
    GraphTopologyNovelty, HammingDistance, NoveltySearch, Op, Tree, distance,
    fitness::{FitnessDescriptor, Novelty},
};

use crate::{ObjectValue, PyEngineInput};

// pub struct PyNovelty {
//     pub descriptor: Option<Py<PyAny>>,
//     pub novelty: NoveltyInner,
// }

pub enum NoveltyInner {
    IntVec(NoveltySearch<Vec<i32>>),
    IntMatrix(NoveltySearch<Vec<Vec<i32>>>),
    FloatVec(NoveltySearch<Vec<f32>>),
    FloatMatrix(NoveltySearch<Vec<Vec<f32>>>),
    CharVec(NoveltySearch<Vec<char>>),
    CharMatrix(NoveltySearch<Vec<Vec<char>>>),
    BitVec(NoveltySearch<Vec<bool>>),
    BitMatrix(NoveltySearch<Vec<Vec<bool>>>),
    GraphTopology(NoveltySearch<Graph<Op<f32>>>),
    TreeTopology(NoveltySearch<Vec<Tree<Op<f32>>>>),
}

pub struct PyDescriptor;

impl Novelty<Vec<f32>> for PyDescriptor {
    fn description(&self, desc: &Vec<f32>) -> Vec<f32> {
        desc.clone()
    }
}

#[pyclass]
pub struct PyNoveltySearch {
    descriptor: Py<PyAny>,
    inner: NoveltySearch<Vec<f32>>,
}

#[pymethods]
impl PyNoveltySearch {
    #[new]
    pub fn new(
        descriptor: Py<PyAny>,
        k: usize,
        threshold: f32,
        archive_size: usize,
        distance: String,
    ) -> Self {
        let mut search =
            NoveltySearch::new(PyDescriptor, k, threshold).with_max_archive_size(archive_size);

        if distance == "EuclideanDistance" {
            search = search.euclidean_distance();
        } else if distance == "CosineDistance" {
            search = search.cosine_distance();
        } else {
            search = search.hamming_distance();
        }

        PyNoveltySearch {
            descriptor,
            inner: search,
        }
    }

    pub fn __call__<'py>(&self, py: Python<'py>, genotype: Py<PyAny>) -> PyResult<f32> {
        let described = self.descriptor.bind(py).call1((genotype,))?;

        if let Ok(vals) = described.extract::<Vec<f32>>() {
            Ok(self.inner.evaluate(&vals))
        } else if let Ok(vals) = described.extract::<Vec<i32>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(&vals))
        } else if let Ok(vals) = described.extract::<Vec<f64>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(&vals))
        } else if let Ok(vals) = described.extract::<Vec<usize>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(&vals))
        } else {
            Err(PyTypeError::new_err(
                "Descriptor did not return a vector of f32 values",
            ))
        }
    }
}

// #[pyclass]
// pub struct PyNoveltySearch {
//     pub descriptor: Option<Py<PyAny>>,
//     pub novelty: NoveltyInner,
// }

// #[pymethods]
// impl PyNoveltySearch {
//     #[new]
//     pub fn new(
//         descriptor: Option<Py<PyAny>>,
//         distance: String,
//         k: usize,
//         threshold: f32,
//         archive_size: usize,
//     ) -> Self {
//         panic!()
//         // PyNoveltySearch {
//         //     descriptor,
//         //     novelty: match distance.as_str() {
//         //         "EuclideanDistance" => NoveltyInner::Euclidean(
//         //             NoveltySearch::new(EuclideanDistance, k, threshold)
//         //                 .with_max_archive_size(archive_size),
//         //         ),
//         //         "CosineDistance" => NoveltyInner::Cosine(
//         //             NoveltySearch::new(CosineDistance, k, threshold)
//         //                 .with_max_archive_size(archive_size),
//         //         ),
//         //         _ => NoveltyInner::Hamming(
//         //             NoveltySearch::new(HammingDistance, k, threshold)
//         //                 .with_max_archive_size(archive_size),
//         //         ),
//         //     },
//         // }
//     }

//     pub fn __call__<'py>(&self, py: Python<'py>, genotype: Py<PyAny>) -> PyResult<f32> {
//         let described = match &self.descriptor {
//             Some(desc) => desc.bind(py).call1((genotype,))?,
//             None => genotype.into_bound(py),
//         };

//         if let Ok(vals) = described.extract::<Vec<f32>>() {
//             Ok(match &self.novelty {
//                 NoveltyInner::Euclidean(n) => n.evaluate(&vals),
//                 NoveltyInner::Cosine(n) => n.evaluate(&vals),
//                 NoveltyInner::Hamming(n) => n.evaluate(&vals),
//                 _ => {
//                     return Err(PyTypeError::new_err(
//                         "Novelty search not implemented for this type",
//                     ));
//                 }
//             })
//         } else if let Ok(vals) = described.extract::<Vec<i32>>() {
//             let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
//             Ok(match &self.novelty {
//                 NoveltyInner::Euclidean(n) => n.evaluate(&vals),
//                 NoveltyInner::Cosine(n) => n.evaluate(&vals),
//                 NoveltyInner::Hamming(n) => n.evaluate(&vals),
//                 _ => {
//                     return Err(PyTypeError::new_err(
//                         "Novelty search not implemented for this type",
//                     ));
//                 }
//             })
//         } else if let Ok(vals) = described.extract::<Vec<f64>>() {
//             let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
//             Ok(match &self.novelty {
//                 NoveltyInner::Euclidean(n) => n.evaluate(&vals),
//                 NoveltyInner::Cosine(n) => n.evaluate(&vals),
//                 NoveltyInner::Hamming(n) => n.evaluate(&vals),
//                 _ => {
//                     return Err(PyTypeError::new_err(
//                         "Novelty search not implemented for this type",
//                     ));
//                 }
//             })
//         } else if let Ok(vals) = described.extract::<Vec<usize>>() {
//             let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
//             Ok(match &self.novelty {
//                 NoveltyInner::Euclidean(n) => n.evaluate(&vals),
//                 NoveltyInner::Cosine(n) => n.evaluate(&vals),
//                 NoveltyInner::Hamming(n) => n.evaluate(&vals),
//                 _ => {
//                     return Err(PyTypeError::new_err(
//                         "Novelty search not implemented for this type",
//                     ));
//                 }
//             })
//         } else {
//             Err(PyTypeError::new_err(
//                 "Descriptor did not return a vector of f32 values",
//             ))
//         }
//     }
// }
