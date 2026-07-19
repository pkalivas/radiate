use crate::{PyGene, PyGeneType, Wrap, dtype};
use pyo3::{
    Bound, IntoPyObject, PyAny, PyResult, Python,
    exceptions::{PyIndexError, PyTypeError, PyValueError},
    pyclass, pymethods,
    types::PyAnyMethods,
};
use radiate::prelude::*;
use radiate_utils::DataType;
use serde::{Deserialize, Serialize};

macro_rules! match_chromosome {
    ($handle:expr, $epoch:ident => $body:expr) => {{
        use ChromosomeInner::*;
        match &$handle {
            UInt8($epoch) => $body,
            UInt16($epoch) => $body,
            UInt32($epoch) => $body,
            UInt64($epoch) => $body,
            UInt128($epoch) => $body,

            Int8($epoch) => $body,
            Int16($epoch) => $body,
            Int32($epoch) => $body,
            Int64($epoch) => $body,
            Int128($epoch) => $body,

            Float32($epoch) => $body,
            Float64($epoch) => $body,

            Char($epoch) => $body,
            Bit($epoch) => $body,

            Permutation($epoch) => $body,

            Graph32($epoch) => $body,
            Graph64($epoch) => $body,

            Tree32($epoch) => $body,
            Tree64($epoch) => $body,
        }
    }};
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum ChromosomeInner {
    UInt8(IntChromosome<u8>),
    UInt16(IntChromosome<u16>),
    UInt32(IntChromosome<u32>),
    UInt64(IntChromosome<u64>),
    UInt128(IntChromosome<u128>),

    Int8(IntChromosome<i8>),
    Int16(IntChromosome<i16>),
    Int32(IntChromosome<i32>),
    Int64(IntChromosome<i64>),
    Int128(IntChromosome<i128>),

    Float32(FloatChromosome<f32>),
    Float64(FloatChromosome<f64>),

    Bit(BitChromosome),
    Char(CharChromosome),

    Permutation(PermutationChromosome<usize>),

    Graph32(GraphChromosome<Op<f32>>),
    Graph64(GraphChromosome<Op<f64>>),

    Tree32(TreeChromosome<Op<f32>>),
    Tree64(TreeChromosome<Op<f64>>),
}

impl From<ChromosomeInner> for Vec<PyGene> {
    fn from(inner: ChromosomeInner) -> Self {
        match inner {
            ChromosomeInner::UInt8(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::UInt16(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::UInt32(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::UInt64(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::UInt128(chrom) => chrom.into_iter().map(PyGene::from).collect(),

            ChromosomeInner::Int8(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Int16(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Int32(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Int64(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Int128(chrom) => chrom.into_iter().map(PyGene::from).collect(),

            ChromosomeInner::Float32(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Float64(chrom) => chrom.into_iter().map(PyGene::from).collect(),

            ChromosomeInner::Bit(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Char(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Permutation(chrom) => chrom.into_iter().map(PyGene::from).collect(),

            ChromosomeInner::Graph32(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Graph64(chrom) => chrom.into_iter().map(PyGene::from).collect(),

            ChromosomeInner::Tree32(chrom) => chrom.into_iter().map(PyGene::from).collect(),
            ChromosomeInner::Tree64(chrom) => chrom.into_iter().map(PyGene::from).collect(),
        }
    }
}

macro_rules! chromosome_from_genes {
      ($genes:expr, $dtype:expr, { $($dt:ident => $variant:ident($chrom:ty, $gene:ty)),* $(,)? }) => {{
          match $dtype {
              $(
                  DataType::$dt => ChromosomeInner::$variant(
                      <$chrom>::from(
                          $genes.into_iter().map(|g| g.into()).collect::<Vec<$gene>>()
                      )
                  ),
              )*
              other => return Err(PyTypeError::new_err(format!(
                  "cannot construct a chromosome from genes of dtype {:?}", other
              ))),
          }
      }};
  }

macro_rules! impl_into_py_chromosome_inner {
    ($chromosome_type:ty, $varient:ident) => {
        impl From<$chromosome_type> for ChromosomeInner {
            fn from(chromosome: $chromosome_type) -> Self {
                ChromosomeInner::$varient(chromosome)
            }
        }
    };
}

impl_into_py_chromosome_inner!(IntChromosome<u8>, UInt8);
impl_into_py_chromosome_inner!(IntChromosome<u16>, UInt16);
impl_into_py_chromosome_inner!(IntChromosome<u32>, UInt32);
impl_into_py_chromosome_inner!(IntChromosome<u64>, UInt64);
impl_into_py_chromosome_inner!(IntChromosome<u128>, UInt128);

impl_into_py_chromosome_inner!(IntChromosome<i8>, Int8);
impl_into_py_chromosome_inner!(IntChromosome<i16>, Int16);
impl_into_py_chromosome_inner!(IntChromosome<i32>, Int32);
impl_into_py_chromosome_inner!(IntChromosome<i64>, Int64);
impl_into_py_chromosome_inner!(IntChromosome<i128>, Int128);

impl_into_py_chromosome_inner!(FloatChromosome<f32>, Float32);
impl_into_py_chromosome_inner!(FloatChromosome<f64>, Float64);

impl_into_py_chromosome_inner!(BitChromosome, Bit);
impl_into_py_chromosome_inner!(CharChromosome, Char);
impl_into_py_chromosome_inner!(PermutationChromosome<usize>, Permutation);

impl_into_py_chromosome_inner!(GraphChromosome<Op<f32>>, Graph32);
impl_into_py_chromosome_inner!(GraphChromosome<Op<f64>>, Graph64);

impl_into_py_chromosome_inner!(TreeChromosome<Op<f32>>, Tree32);
impl_into_py_chromosome_inner!(TreeChromosome<Op<f64>>, Tree64);

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyChromosome {
    pub(crate) inner: ChromosomeInner,
}

#[pymethods]
impl PyChromosome {
    #[new]
    pub fn new<'py>(py: Python<'py>, genes: Vec<PyGene>) -> PyResult<Self> {
        if genes.is_empty() {
            return Err(PyValueError::new_err(
                "cannot construct a PyChromosome from an empty gene list — gene type is unknown",
            ));
        }

        let gene_type = genes[0].gene_type();

        if !genes.iter().all(|g| g.gene_type() == gene_type) {
            return Err(PyValueError::new_err(
                "all genes in a chromosome must have the same type",
            ));
        }

        if matches!(
            gene_type,
            PyGeneType::Permutation | PyGeneType::GraphNode | PyGeneType::TreeNode
        ) {
            return Err(PyValueError::new_err(
                "cannot construct a PyChromosome from genes of type Permutation, GraphNode, or TreeNode",
            ));
        }

        let first_dtype = genes[0].dtype(py)?.extract::<Wrap<DataType>>()?.0;
        let inner = chromosome_from_genes!(genes, first_dtype, {
            UInt8   => UInt8(IntChromosome<u8>,    IntGene<u8>),
            UInt16  => UInt16(IntChromosome<u16>,  IntGene<u16>),
            UInt32  => UInt32(IntChromosome<u32>,  IntGene<u32>),
            UInt64  => UInt64(IntChromosome<u64>,  IntGene<u64>),
            UInt128 => UInt128(IntChromosome<u128>, IntGene<u128>),
            Int8    => Int8(IntChromosome<i8>,     IntGene<i8>),
            Int16   => Int16(IntChromosome<i16>,   IntGene<i16>),
            Int32   => Int32(IntChromosome<i32>,   IntGene<i32>),
            Int64   => Int64(IntChromosome<i64>,   IntGene<i64>),
            Int128  => Int128(IntChromosome<i128>, IntGene<i128>),
            Float32 => Float32(FloatChromosome<f32>, FloatGene<f32>),
            Float64 => Float64(FloatChromosome<f64>, FloatGene<f64>),
            Boolean => Bit(BitChromosome,          BitGene),
            Char    => Char(CharChromosome,        CharGene),
        });

        Ok(PyChromosome { inner })
    }

    pub fn __repr__(&self) -> String {
        match_chromosome!(self.inner, chrom => {
            format!("{:?}", chrom)
        })
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        match_chromosome!(self.inner, chrom => chrom.len())
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    pub fn __getitem__(&self, index: isize) -> PyResult<PyGene> {
        let len = match_chromosome!(self.inner, chrom => chrom.len());
        if index >= len as isize || index < -(len as isize) {
            return Err(PyIndexError::new_err("index out of range"));
        }

        let index = if index < 0 {
            index + len as isize
        } else {
            index
        } as usize;

        match_chromosome!(self.inner, chrom => match chrom.get(index) {
            Some(gene) => Ok(PyGene::from(gene.clone())),
            None => Err(PyIndexError::new_err("index out of range")),
        })
    }

    pub fn gene_type(&self) -> PyGeneType {
        match self.inner {
            ChromosomeInner::UInt8(_) => PyGeneType::Int,
            ChromosomeInner::UInt16(_) => PyGeneType::Int,
            ChromosomeInner::UInt32(_) => PyGeneType::Int,
            ChromosomeInner::UInt64(_) => PyGeneType::Int,
            ChromosomeInner::UInt128(_) => PyGeneType::Int,

            ChromosomeInner::Int8(_) => PyGeneType::Int,
            ChromosomeInner::Int16(_) => PyGeneType::Int,
            ChromosomeInner::Int32(_) => PyGeneType::Int,
            ChromosomeInner::Int64(_) => PyGeneType::Int,
            ChromosomeInner::Int128(_) => PyGeneType::Int,

            ChromosomeInner::Float32(_) => PyGeneType::Float,
            ChromosomeInner::Float64(_) => PyGeneType::Float,

            ChromosomeInner::Bit(_) => PyGeneType::Bit,
            ChromosomeInner::Char(_) => PyGeneType::Char,

            ChromosomeInner::Permutation(_) => PyGeneType::Permutation,

            ChromosomeInner::Graph32(_) => PyGeneType::GraphNode,
            ChromosomeInner::Graph64(_) => PyGeneType::GraphNode,

            ChromosomeInner::Tree32(_) => PyGeneType::TreeNode,
            ChromosomeInner::Tree64(_) => PyGeneType::TreeNode,
        }
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dtype = match self.inner {
            ChromosomeInner::UInt8(_) => DataType::UInt8,
            ChromosomeInner::UInt16(_) => DataType::UInt16,
            ChromosomeInner::UInt32(_) => DataType::UInt32,
            ChromosomeInner::UInt64(_) => DataType::UInt64,
            ChromosomeInner::UInt128(_) => DataType::UInt128,

            ChromosomeInner::Int8(_) => DataType::Int8,
            ChromosomeInner::Int16(_) => DataType::Int16,
            ChromosomeInner::Int32(_) => DataType::Int32,
            ChromosomeInner::Int64(_) => DataType::Int64,
            ChromosomeInner::Int128(_) => DataType::Int128,

            ChromosomeInner::Float32(_) => DataType::Float32,
            ChromosomeInner::Float64(_) => DataType::Float64,

            ChromosomeInner::Bit(_) => DataType::Boolean,
            ChromosomeInner::Char(_) => DataType::Char,

            ChromosomeInner::Permutation(_) => DataType::Usize,

            ChromosomeInner::Graph32(_) => dtype::graph_node_dtype(DataType::Float32),
            ChromosomeInner::Graph64(_) => dtype::graph_node_dtype(DataType::Float64),

            ChromosomeInner::Tree32(_) => dtype::tree_node_dtype(DataType::Float32),
            ChromosomeInner::Tree64(_) => dtype::tree_node_dtype(DataType::Float64),
        };

        Wrap(dtype).into_pyobject(py)
    }
}

macro_rules! impl_into_py_chromosome {
    ($chromosome_type:ty, $variant:ident) => {
        impl From<$chromosome_type> for PyChromosome {
            fn from(chromosome: $chromosome_type) -> Self {
                PyChromosome {
                    inner: ChromosomeInner::from(chromosome),
                }
            }
        }

        impl From<PyChromosome> for $chromosome_type {
            fn from(py_chromosome: PyChromosome) -> Self {
                match py_chromosome.inner {
                    ChromosomeInner::$variant(chrom) => chrom,
                    _ => panic!("Invalid chromosome type for conversion"),
                }
            }
        }
    };
}

impl_into_py_chromosome!(IntChromosome<u8>, UInt8);
impl_into_py_chromosome!(IntChromosome<u16>, UInt16);
impl_into_py_chromosome!(IntChromosome<u32>, UInt32);
impl_into_py_chromosome!(IntChromosome<u64>, UInt64);
impl_into_py_chromosome!(IntChromosome<u128>, UInt128);

impl_into_py_chromosome!(IntChromosome<i8>, Int8);
impl_into_py_chromosome!(IntChromosome<i16>, Int16);
impl_into_py_chromosome!(IntChromosome<i32>, Int32);
impl_into_py_chromosome!(IntChromosome<i64>, Int64);
impl_into_py_chromosome!(IntChromosome<i128>, Int128);

impl_into_py_chromosome!(FloatChromosome<f32>, Float32);
impl_into_py_chromosome!(FloatChromosome<f64>, Float64);

impl_into_py_chromosome!(BitChromosome, Bit);
impl_into_py_chromosome!(CharChromosome, Char);
impl_into_py_chromosome!(PermutationChromosome<usize>, Permutation);

impl_into_py_chromosome!(GraphChromosome<Op<f32>>, Graph32);
impl_into_py_chromosome!(GraphChromosome<Op<f64>>, Graph64);

impl_into_py_chromosome!(TreeChromosome<Op<f32>>, Tree32);
impl_into_py_chromosome!(TreeChromosome<Op<f64>>, Tree64);
