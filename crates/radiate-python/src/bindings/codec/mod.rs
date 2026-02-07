mod any;
mod bit;
mod char;
mod float;
mod graph;
mod int;
mod permutation;
mod tree;

use std::sync::Arc;

pub use any::PyAnyCodec;
pub use bit::PyBitCodec;
pub use char::PyCharCodec;
pub use float::PyFloatCodec;
pub use graph::PyGraphCodec;
pub use int::PyIntCodec;
pub use permutation::PyPermutationCodec;
pub use tree::PyTreeCodec;

use numpy::{Element, PyArray, PyArray1, PyArrayMethods};
use pyo3::Python;
use pyo3::{
    Bound, IntoPyObject, PyAny, PyResult,
    types::{PyList, PyListMethods},
};
use radiate::{Chromosome, Codec, Gene, Genotype};

#[derive(Clone)]
pub struct PyCodec<C: Chromosome, T> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<Arc<dyn for<'py> Fn(Python<'py>, &Genotype<C>) -> T + Send + Sync>>,
}

impl<C: Chromosome, T> PyCodec<C, T> {
    pub fn new() -> Self {
        PyCodec {
            encoder: None,
            decoder: None,
        }
    }

    pub fn decode_with_py(&self, py: Python<'_>, genotype: &Genotype<C>) -> T {
        match &self.decoder {
            Some(decoder) => decoder(py, genotype),
            None => panic!("Decoder function is not set"),
        }
    }

    pub fn with_encoder<F>(mut self, encoder: F) -> Self
    where
        F: Fn() -> Genotype<C> + 'static,
    {
        self.encoder = Some(Arc::new(encoder));
        self
    }

    pub fn with_decoder<F>(mut self, decoder: F) -> Self
    where
        F: for<'py> Fn(Python<'py>, &Genotype<C>) -> T + 'static + Send + Sync,
    {
        self.decoder = Some(Arc::new(decoder));
        self
    }
}

impl<C: Chromosome, T> Codec<C, T> for PyCodec<C, T> {
    fn encode(&self) -> Genotype<C> {
        match &self.encoder {
            Some(encoder) => encoder(),
            None => panic!("Encoder function is not set"),
        }
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        Python::attach(|py| match &self.decoder {
            Some(decoder) => decoder(py, genotype),
            None => panic!("Decoder function is not set"),
        })
    }
}

unsafe impl<C: Chromosome, T> Send for PyCodec<C, T> {}
unsafe impl<C: Chromosome, T> Sync for PyCodec<C, T> {}

pub(super) fn decode_genotype_to_array<'py, C, G, A>(
    py: Python<'py>,
    genotype: &Genotype<C>,
    use_numpy: bool,
) -> PyResult<Bound<'py, PyAny>>
where
    C: Chromosome<Gene = G>,
    G: Gene<Allele = A>,
    A: Element + IntoPyObject<'py> + Copy,
{
    let lengths = genotype
        .iter()
        .map(|chrom| chrom.len())
        .collect::<Vec<usize>>();

    if genotype.len() == 1 {
        let values = genotype
            .iter()
            .next()
            .map(|chrom| chrom.iter().map(|gene| *gene.allele()).collect::<Vec<A>>())
            .unwrap_or_default();

        let is_square = lengths.iter().all(|&len| len == lengths[0]);

        if is_square && use_numpy {
            return match lengths.len() {
                1 => Ok(PyArray1::from_vec(py, values).into_any()),
                _ => Ok(PyArray::from_iter(py, values)
                    .reshape([lengths.len(), lengths[0]])?
                    .into_any()),
            };
        }

        return Ok(PyList::new(py, values)?.into_any());
    }

    let is_square = lengths.iter().all(|&len| len == lengths[0]);

    if use_numpy && is_square {
        let values = genotype
            .iter()
            .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
            .collect::<Vec<A>>();

        return match lengths.len() {
            1 => Ok(PyArray1::from_vec(py, values).into_any()),
            _ => Ok(PyArray::from_iter(py, values)
                .reshape([lengths.len(), lengths[0]])?
                .into_any()),
        };
    }

    let values = genotype
        .iter()
        .map(|chrom| chrom.iter().map(|gene| *gene.allele()).collect::<Vec<A>>())
        .collect::<Vec<Vec<A>>>();

    let outer = PyList::empty(py);
    for chromo in values.iter() {
        let inner = PyList::empty(py);
        for gene in chromo.iter() {
            inner.append(*gene).unwrap();
        }
        outer.append(inner).unwrap();
    }

    Ok(outer.into_any())
}

// use numpy::{Element, PyArray, PyArray1, PyArrayMethods};
// use pyo3::{
//     Bound, PyAny, PyResult, Python,
//     exceptions::PyValueError,
//     types::{PyList, PyListMethods},
// };
// use radiate::{Chromosome, Gene, Genotype};

// /// Decode a genotype into either:
// /// - a numeric numpy ndarray (rectangular; supports N-D via `reshape`)
// /// - a numpy object array (jagged; if `jagged_numpy=true`)
// /// - python lists (if `use_numpy=false`)
// ///
// /// Rules:
// /// 1) If `use_numpy && reshape.is_some()` => flatten + reshape to N-D numeric ndarray (must match element count).
// /// 2) Else if `use_numpy` and rectangular => 1D/2D numeric ndarray (1 chrom => 1D, N chrom => 2D).
// /// 3) Else if `use_numpy` and not rectangular:
// ///      - if `jagged_numpy` => np.array(list_of_inner_arrays, dtype=object)
// ///      - else => list-of-lists
// /// 4) If `use_numpy=false` => list or list-of-lists
// pub(super) fn decode_genotype_to_array<'py, C, G, A>(
//     py: Python<'py>,
//     genotype: &Genotype<C>,
//     use_numpy: bool,
//     reshape: Option<Vec<usize>>,
//     jagged_numpy: bool,
// ) -> PyResult<Bound<'py, PyAny>>
// where
//     C: Chromosome<Gene = G>,
//     G: Gene<Allele = A>,
//     A: Element + Copy,
// {
//     // --- collect basic shape stats ---
//     let chrom_count = genotype.len();
//     let lengths: Vec<usize> = genotype.iter().map(|c| c.len()).collect();

//     let rectangular = match lengths.split_first() {
//         None => true, // empty genotype => treat as rectangular
//         Some((&first, rest)) => rest.iter().all(|&n| n == first),
//     };

//     // Total allele count
//     let total_len: usize = lengths.iter().sum();

//     // --- Case 1: explicit N-D reshape (numeric ndarray) ---
//     if use_numpy {
//         if let Some(shape) = reshape {
//             let expected: usize = shape.iter().product();
//             if expected != total_len {
//                 return Err(PyValueError::new_err(format!(
//                     "reshape {:?} requires {} elements, but genotype has {}",
//                     shape, expected, total_len
//                 )));
//             }

//             // Flatten and reshape
//             let flat: Vec<A> = genotype
//                 .iter()
//                 .flat_map(|chrom| chrom.iter().map(|g| *g.allele()))
//                 .collect();

//             // Build 1D then reshape via numpy (easiest, avoids fixed-size reshape signatures)
//             let arr1 = PyArray1::from_vec(py, flat);
//             let np = py.import("numpy")?;
//             let reshaped = np.call_method1("reshape", (arr1, shape))?;
//             return Ok(reshaped.into_bound());
//         }
//     }

//     // --- Case 2: no explicit reshape; default 1D/2D behavior ---
//     if chrom_count == 0 {
//         // Empty genotype
//         if use_numpy {
//             // empty 1D numeric array
//             let arr = PyArray1::<A>::from_vec(py, Vec::new()).into_any();
//             return Ok(arr);
//         }
//         return Ok(PyList::empty(py).into_any());
//     }

//     // Single chromosome => 1D
//     if chrom_count == 1 {
//         let values: Vec<A> = genotype
//             .iter()
//             .next()
//             .unwrap()
//             .iter()
//             .map(|g| *g.allele())
//             .collect();

//         if use_numpy {
//             return Ok(PyArray1::from_vec(py, values).into_any());
//         } else {
//             return Ok(PyList::new(py, values)?.into_any());
//         }
//     }

//     // Multiple chromosomes:
//     // Rectangular => numeric 2D ndarray
//     if use_numpy && rectangular {
//         let cols = lengths[0];
//         let rows = chrom_count;

//         let flat: Vec<A> = genotype
//             .iter()
//             .flat_map(|chrom| chrom.iter().map(|g| *g.allele()))
//             .collect();

//         // Create 1D then reshape to (rows, cols)
//         let arr1 = PyArray1::from_vec(py, flat);
//         let arr2 = arr1.reshape([rows, cols])?;
//         return Ok(arr2.into_any());
//     }

//     // Not rectangular:
//     // - If jagged_numpy => numpy object array: elements are inner 1D arrays
//     // - Else => list-of-lists
//     if use_numpy && jagged_numpy {
//         let np = py.import("numpy")?;
//         let outer = PyList::empty(py);

//         for chrom in genotype.iter() {
//             let inner: Vec<A> = chrom.iter().map(|g| *g.allele()).collect();
//             let inner_arr = PyArray1::from_vec(py, inner).into_any();
//             outer.append(inner_arr)?;
//         }

//         // np.array(outer, dtype=object)
//         let obj = np.getattr("object_")?;
//         let obj_arr = np.call_method1("array", (outer,))?
//             .call_method1("astype", (obj,))?;
//         return Ok(obj_arr.into_bound());
//     }

//     // Fallback: pure Python nested lists
//     let outer = PyList::empty(py);
//     for chrom in genotype.iter() {
//         let inner = PyList::empty(py);
//         for gene in chrom.iter() {
//             inner.append(*gene.allele())?;
//         }
//         outer.append(inner)?;
//     }
//     Ok(outer.into_any())
// }
