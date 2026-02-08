use crate::{AnyGene, AnyValue, PyGeneType, Wrap, bindings::dtype};
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitGene, CharGene, DataType, Float, FloatGene, Gene, GraphNode, IntGene, Integer, Op,
    PermutationGene, TreeNode, dtype_names, random_provider,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum GeneInner {
    UInt8(IntGene<u8>),
    UInt16(IntGene<u16>),
    UInt32(IntGene<u32>),
    UInt64(IntGene<u64>),
    UInt128(IntGene<u128>),
    Int8(IntGene<i8>),
    Int16(IntGene<i16>),
    Int32(IntGene<i32>),
    Int64(IntGene<i64>),
    Int128(IntGene<i128>),

    Float32(FloatGene<f32>),
    Float64(FloatGene<f64>),

    Bit(BitGene),
    Char(CharGene),
    Permutation(PermutationGene<usize>),
    GraphNode(GraphNode<Op<f32>>),
    TreeNode(TreeNode<Op<f32>>),
    AnyGene(AnyGene<'static>),
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub struct PyGene {
    inner: GeneInner,
}

#[pymethods]
impl PyGene {
    pub fn __str__(&self) -> String {
        match &self.inner {
            GeneInner::UInt8(gene) => format!("{}", gene),
            GeneInner::UInt16(gene) => format!("{}", gene),
            GeneInner::UInt32(gene) => format!("{}", gene),
            GeneInner::UInt64(gene) => format!("{}", gene),
            GeneInner::UInt128(gene) => format!("{}", gene),

            GeneInner::Int8(gene) => format!("{}", gene),
            GeneInner::Int16(gene) => format!("{}", gene),
            GeneInner::Int32(gene) => format!("{}", gene),
            GeneInner::Int64(gene) => format!("{}", gene),
            GeneInner::Int128(gene) => format!("{}", gene),

            GeneInner::Float32(gene) => format!("{}", gene),
            GeneInner::Float64(gene) => format!("{}", gene),

            GeneInner::Bit(gene) => format!("{}", gene),
            GeneInner::Char(gene) => format!("{:?}", gene),
            GeneInner::GraphNode(gene) => format!("{:?}", gene),
            GeneInner::TreeNode(gene) => format!("{:?}", gene),
            GeneInner::AnyGene(gene) => format!("{:?}", gene),
            GeneInner::Permutation(gene) => format!("{:?}", gene),
        }
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    pub fn gene_type(&self) -> PyGeneType {
        match &self.inner {
            GeneInner::UInt8(_) => PyGeneType::Int,
            GeneInner::UInt16(_) => PyGeneType::Int,
            GeneInner::UInt32(_) => PyGeneType::Int,
            GeneInner::UInt64(_) => PyGeneType::Int,
            GeneInner::UInt128(_) => PyGeneType::Int,

            GeneInner::Int8(_) => PyGeneType::Int,
            GeneInner::Int16(_) => PyGeneType::Int,
            GeneInner::Int32(_) => PyGeneType::Int,
            GeneInner::Int64(_) => PyGeneType::Int,
            GeneInner::Int128(_) => PyGeneType::Int,

            GeneInner::Float32(_) => PyGeneType::Float,
            GeneInner::Float64(_) => PyGeneType::Float,

            GeneInner::Bit(_) => PyGeneType::Bit,
            GeneInner::Char(_) => PyGeneType::Char,
            GeneInner::GraphNode(_) => PyGeneType::GraphNode,
            GeneInner::TreeNode(_) => PyGeneType::TreeNode,
            GeneInner::Permutation(_) => PyGeneType::Permutation,
            GeneInner::AnyGene(_) => PyGeneType::AnyGene,
        }
    }

    pub fn dtype(&self) -> String {
        match &self.inner {
            GeneInner::UInt8(_) => dtype_names::UINT8.into(),
            GeneInner::UInt16(_) => dtype_names::UINT16.into(),
            GeneInner::UInt32(_) => dtype_names::UINT32.into(),
            GeneInner::UInt64(_) => dtype_names::UINT64.into(),
            GeneInner::UInt128(_) => dtype_names::UINT128.into(),

            GeneInner::Int8(_) => dtype_names::INT8.into(),
            GeneInner::Int16(_) => dtype_names::INT16.into(),
            GeneInner::Int32(_) => dtype_names::INT32.into(),
            GeneInner::Int64(_) => dtype_names::INT64.into(),
            GeneInner::Int128(_) => dtype_names::INT128.into(),

            GeneInner::Float32(_) => dtype_names::FLOAT32.into(),
            GeneInner::Float64(_) => dtype_names::FLOAT64.into(),

            GeneInner::Bit(_) => dtype_names::BOOLEAN.into(),
            GeneInner::Char(_) => dtype_names::CHAR.into(),
            GeneInner::GraphNode(_) => dtype_names::STRUCT.into(),
            GeneInner::TreeNode(_) => dtype_names::STRUCT.into(),
            GeneInner::Permutation(_) => dtype_names::USIZE.into(),
            GeneInner::AnyGene(_) => dtype_names::STRUCT.into(),
        }
    }

    pub fn allele<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            GeneInner::UInt8(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::UInt16(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::UInt32(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::UInt64(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::UInt128(gene) => gene.allele().into_bound_py_any(py),

            GeneInner::Int8(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int16(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int32(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int64(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int128(gene) => gene.allele().into_bound_py_any(py),

            GeneInner::Float32(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Float64(gene) => gene.allele().into_bound_py_any(py),

            GeneInner::Bit(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Char(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::GraphNode(gene) => gene.allele().name().into_bound_py_any(py),
            GeneInner::TreeNode(gene) => gene.allele().name().into_bound_py_any(py),
            GeneInner::Permutation(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::AnyGene(gene) => Wrap(gene.allele()).into_bound_py_any(py),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, range=None, bounds=None, dtype=None))]
    pub fn float(
        allele: Option<f64>,
        range: Option<(f64, f64)>,
        bounds: Option<(f64, f64)>,
        dtype: Option<String>,
    ) -> PyGene {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::FLOAT64.into()));
        let range = range.unwrap_or((std::f64::MIN, std::f64::MAX));
        let bounds = bounds.unwrap_or(range.clone());

        fn to_gene<F: Float>(
            allele: Option<f64>,
            range: (f64, f64),
            bounds: (f64, f64),
        ) -> FloatGene<F> {
            match allele {
                Some(a) => FloatGene::new(
                    F::from_f64(a),
                    F::from_f64(range.0)..F::from_f64(range.1),
                    F::from_f64(bounds.0)..F::from_f64(bounds.1),
                ),
                None => FloatGene::from((
                    F::from_f64(range.0)..F::from_f64(range.1),
                    F::from_f64(bounds.0)..F::from_f64(bounds.1),
                )),
            }
        }

        PyGene {
            inner: match dtype {
                DataType::Float32 => GeneInner::Float32(to_gene::<f32>(allele, range, bounds)),
                DataType::Float64 => GeneInner::Float64(to_gene::<f64>(allele, range, bounds)),
                _ => panic!("Unsupported float dtype: {:?}", dtype),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, range=None, bounds=None, dtype=None))]
    pub fn int(
        allele: Option<i64>,
        range: Option<(i64, i64)>,
        bounds: Option<(i64, i64)>,
        dtype: Option<String>,
    ) -> PyGene {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::INT64.into()));
        let range = range.unwrap_or((i64::MIN, i64::MAX));
        let bounds = bounds.unwrap_or(range.clone());

        fn to_gene<I: Integer>(
            allele: Option<i64>,
            range: (i64, i64),
            bounds: (i64, i64),
        ) -> IntGene<I> {
            match allele {
                Some(a) => IntGene::new(
                    I::from_i64(a),
                    I::from_i64(range.0)..I::from_i64(range.1),
                    I::from_i64(bounds.0)..I::from_i64(bounds.1),
                ),
                None => IntGene::from((
                    I::from_i64(range.0)..I::from_i64(range.1),
                    I::from_i64(bounds.0)..I::from_i64(bounds.1),
                )),
            }
        }

        PyGene {
            inner: match dtype {
                DataType::UInt8 => GeneInner::UInt8(to_gene::<u8>(allele, range, bounds)),
                DataType::UInt16 => GeneInner::UInt16(to_gene::<u16>(allele, range, bounds)),
                DataType::UInt32 => GeneInner::UInt32(to_gene::<u32>(allele, range, bounds)),
                DataType::UInt64 => GeneInner::UInt64(to_gene::<u64>(allele, range, bounds)),
                DataType::UInt128 => GeneInner::UInt128(to_gene::<u128>(allele, range, bounds)),
                DataType::Int8 => GeneInner::Int8(to_gene::<i8>(allele, range, bounds)),
                DataType::Int16 => GeneInner::Int16(to_gene::<i16>(allele, range, bounds)),
                DataType::Int32 => GeneInner::Int32(to_gene::<i32>(allele, range, bounds)),
                DataType::Int64 => GeneInner::Int64(to_gene::<i64>(allele, range, bounds)),
                DataType::Int128 => GeneInner::Int128(to_gene::<i128>(allele, range, bounds)),
                _ => panic!("Unsupported integer dtype: {:?}", dtype),
            },
        }
    }

    #[staticmethod]
    pub fn bit(allele: Option<bool>) -> PyGene {
        PyGene {
            inner: GeneInner::Bit(BitGene::from(allele.unwrap_or(random_provider::bool(0.5)))),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, char_set=None))]
    pub fn char(allele: Option<char>, char_set: Option<Vec<char>>) -> PyGene {
        PyGene {
            inner: GeneInner::Char(match char_set {
                Some(chars) => match allele {
                    Some(a) => CharGene::from((a, chars.into_iter().collect())),
                    None => CharGene::new(chars.into_iter().collect()),
                },
                None => match allele {
                    Some(a) => CharGene::from(a),
                    None => CharGene::default(),
                },
            }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele, metadata, factory))]
    pub fn any(
        allele: Wrap<AnyValue<'_>>,
        metadata: HashMap<String, String>,
        factory: Py<PyAny>,
    ) -> PyGene {
        let fact = move || {
            Python::attach(|py| {
                let obj = factory.call0(py).unwrap();
                let gene = obj.extract::<Wrap<AnyValue<'_>>>(py).unwrap();
                gene.0.into_static()
            })
        };

        PyGene {
            inner: GeneInner::AnyGene(
                AnyGene::new(allele.0.into_static())
                    .with_metadata(metadata)
                    .with_factory(fact),
            ),
        }
    }
}

macro_rules! impl_into_py_gene {
    ($gene_type:ty, $gene_variant:ident) => {
        impl From<$gene_type> for PyGene {
            fn from(gene: $gene_type) -> Self {
                PyGene {
                    inner: GeneInner::$gene_variant(gene),
                }
            }
        }

        impl From<PyGene> for $gene_type {
            fn from(py_gene: PyGene) -> Self {
                match py_gene.inner {
                    GeneInner::$gene_variant(gene) => gene,
                    _ => panic!("Cannot convert PyGene to {}", stringify!($gene_type)),
                }
            }
        }
    };
}

impl_into_py_gene!(IntGene<u8>, UInt8);
impl_into_py_gene!(IntGene<u16>, UInt16);
impl_into_py_gene!(IntGene<u32>, UInt32);
impl_into_py_gene!(IntGene<u64>, UInt64);
impl_into_py_gene!(IntGene<u128>, UInt128);

impl_into_py_gene!(IntGene<i8>, Int8);
impl_into_py_gene!(IntGene<i16>, Int16);
impl_into_py_gene!(IntGene<i32>, Int32);
impl_into_py_gene!(IntGene<i64>, Int64);
impl_into_py_gene!(IntGene<i128>, Int128);

impl_into_py_gene!(FloatGene<f32>, Float32);
impl_into_py_gene!(FloatGene<f64>, Float64);

impl_into_py_gene!(BitGene, Bit);
impl_into_py_gene!(CharGene, Char);
impl_into_py_gene!(GraphNode<Op<f32>>, GraphNode);
impl_into_py_gene!(TreeNode<Op<f32>>, TreeNode);
impl_into_py_gene!(PermutationGene<usize>, Permutation);
impl_into_py_gene!(AnyGene<'static>, AnyGene);
