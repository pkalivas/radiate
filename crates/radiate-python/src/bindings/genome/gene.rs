use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitGene, CharGene, FloatGene, Gene, GraphNode, IntGene, Op, PermutationGene, TreeNode,
    random_provider,
};

use crate::PyGeneType;

#[derive(Clone, Debug, PartialEq)]
enum GeneInner {
    Float(FloatGene),
    Int(IntGene<i32>),
    Bit(BitGene),
    Char(CharGene),
    GraphNode(GraphNode<Op<f32>>),
    TreeNode(TreeNode<Op<f32>>),
    Permutation(PermutationGene<usize>),
    Empty(PyGeneType),
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct PyGene {
    inner: GeneInner,
}

#[pymethods]
impl PyGene {
    pub fn __str__(&self) -> String {
        match &self.inner {
            GeneInner::Float(gene) => format!("{}", gene),
            GeneInner::Int(gene) => format!("{}", gene),
            GeneInner::Bit(gene) => format!("{}", gene),
            GeneInner::Char(gene) => format!("{:?}", gene),
            GeneInner::GraphNode(gene) => format!("{:?}", gene),
            GeneInner::TreeNode(gene) => format!("{:?}", gene),
            GeneInner::Permutation(gene) => format!("{:?}", gene),
            GeneInner::Empty(gene_type) => format!("Empty({})", gene_type.name()),
        }
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    pub fn gene_type(&self) -> PyGeneType {
        match &self.inner {
            GeneInner::Float(_) => PyGeneType::Float,
            GeneInner::Int(_) => PyGeneType::Int,
            GeneInner::Bit(_) => PyGeneType::Bit,
            GeneInner::Char(_) => PyGeneType::Char,
            GeneInner::GraphNode(_) => PyGeneType::GraphNode,
            GeneInner::TreeNode(_) => PyGeneType::TreeNode,
            GeneInner::Permutation(_) => PyGeneType::Permutation,
            GeneInner::Empty(gene_type) => gene_type.clone(),
        }
    }

    pub fn allele<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            GeneInner::Float(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Bit(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Char(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::GraphNode(gene) => gene.allele().name().into_bound_py_any(py),
            GeneInner::TreeNode(gene) => gene.allele().name().into_bound_py_any(py),
            GeneInner::Permutation(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Empty(_) => py.None().into_bound_py_any(py),
        }
    }

    pub fn with_allele<'py>(&self, py: Python<'py>, allele: Option<Py<PyAny>>) -> PyResult<PyGene> {
        if allele.is_none() {
            return Ok(PyGene {
                inner: match &self.inner {
                    GeneInner::Float(gene) => GeneInner::Float(gene.new_instance()),
                    GeneInner::Int(gene) => GeneInner::Int(gene.new_instance()),
                    GeneInner::Bit(gene) => GeneInner::Bit(gene.new_instance()),
                    GeneInner::Char(gene) => GeneInner::Char(gene.new_instance()),
                    GeneInner::Permutation(gene) => GeneInner::Permutation(gene.new_instance()),
                    _ => {
                        return Err(pyo3::exceptions::PyTypeError::new_err(
                            "Cannot set allele on this gene type",
                        ));
                    }
                },
            });
        } else if let Some(allele) = allele {
            return Ok(PyGene {
                inner: match &self.inner {
                    GeneInner::Float(gene) => {
                        GeneInner::Float(gene.with_allele(&allele.extract(py)?))
                    }
                    GeneInner::Int(gene) => GeneInner::Int(gene.with_allele(&allele.extract(py)?)),
                    GeneInner::Bit(gene) => GeneInner::Bit(gene.with_allele(&allele.extract(py)?)),
                    GeneInner::Char(gene) => {
                        GeneInner::Char(gene.with_allele(&allele.extract(py)?))
                    }
                    GeneInner::Permutation(gene) => {
                        GeneInner::Permutation(gene.with_allele(&allele.extract(py)?))
                    }
                    _ => {
                        return Err(pyo3::exceptions::PyTypeError::new_err(
                            "Cannot set allele on this gene type",
                        ));
                    }
                },
            });
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Allele must be of the correct type",
            ))
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, range=None, bounds=None))]
    pub fn float(
        allele: Option<f32>,
        range: Option<(f32, f32)>,
        bounds: Option<(f32, f32)>,
    ) -> PyGene {
        let range = range.unwrap_or((std::f32::MIN, std::f32::MAX));
        let bounds = bounds.unwrap_or(range.clone());

        PyGene {
            inner: GeneInner::Float(match allele {
                Some(a) => FloatGene::new(a, range.0..range.1, bounds.0..bounds.1),
                None => FloatGene::from((range.0..range.1, bounds.0..bounds.1)),
            }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, range=None, bounds=None))]
    pub fn int(
        allele: Option<i32>,
        range: Option<(i32, i32)>,
        bounds: Option<(i32, i32)>,
    ) -> PyGene {
        let range = range.unwrap_or((i32::MIN, i32::MAX));
        let bounds = bounds.unwrap_or(range.clone());

        PyGene {
            inner: GeneInner::Int(match allele {
                Some(a) => IntGene::new(a, range.0..range.1, bounds.0..bounds.1),
                None => IntGene::from((range.0..range.1, bounds.0..bounds.1)),
            }),
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
    pub fn empty(gene_type: PyGeneType) -> PyGene {
        PyGene {
            inner: GeneInner::Empty(gene_type),
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

impl_into_py_gene!(FloatGene, Float);
impl_into_py_gene!(IntGene<i32>, Int);
impl_into_py_gene!(BitGene, Bit);
impl_into_py_gene!(CharGene, Char);
impl_into_py_gene!(GraphNode<Op<f32>>, GraphNode);
impl_into_py_gene!(TreeNode<Op<f32>>, TreeNode);
impl_into_py_gene!(PermutationGene<usize>, Permutation);
