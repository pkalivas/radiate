use crate::{AnyChromosome, AnyGene, PyAnyObject, PyCodec, PyGene, PyGenotype, prelude::Wrap};
use crate::{AnyValue, Field};
use pyo3::{
    Borrowed, FromPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyResult, Python, pyclass,
    pymethods,
    types::{PyAnyMethods, PyList},
};
use radiate::{Chromosome, Codec, DataType, Gene, Genotype};
use radiate::{FloatChromosome, FloatGene};
use radiate_utils::{Float, Primitive, SmallStr};
use std::{ops::Range, sync::Arc};

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyFieldCodec {
    pub codec: PyCodec<FloatChromosome<f64>, PyAnyObject>,
}

#[pymethods]
impl PyFieldCodec {
    #[new]
    // pub fn new(genes: Vec<PyGene>, creator: Py<PyAny>) -> PyResult<Self> {
    pub fn new(count: usize, specs: PyFieldSpec) -> PyResult<Self> {
        let cloned_specs = specs.clone();

        Ok(PyFieldCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    (0..count)
                        .map(|_| encode_mixed(&cloned_specs.spec))
                        .collect::<Genotype<FloatChromosome<f64>>>()
                })
                .with_decoder(move |py, genotype| {
                    let any_values = genotype
                        .iter()
                        .map(|chromo| {
                            let mut offset = 0;
                            decode_plan(&specs.spec, chromo.as_slice(), &mut offset)
                        })
                        .collect::<Vec<AnyValue<'static>>>();
                    PyAnyObject {
                        inner: PyList::new(
                            py,
                            any_values.into_iter().map(Wrap).collect::<Vec<_>>(),
                        )
                        .unwrap()
                        .unbind()
                        .into_any(),
                    }
                }),
        })

        // todo!()

        // let call_creator = move |py: Python<'_>, allele: &AnyGene| -> PyResult<PyAnyObject> {
        //     let obj = creator.call1(
        //         py,
        //         (Wrap(allele.allele()).into_py_any(py)?, allele.metadata()),
        //     )?;

        //     Ok(PyAnyObject {
        //         inner: obj.into_any(),
        //     })
        // };

        // let temp = genes
        //     .iter()
        //     .map(|v| AnyGene::from(v.clone()))
        //     .collect::<AnyChromosome>();

        // Ok(PyAnyCodec {
        //     codec: PyCodec::new()
        //         .with_encoder(move || {
        //             Genotype::from(
        //                 genes
        //                     .iter()
        //                     .map(|v| AnyGene::from(v.clone()))
        //                     .collect::<AnyChromosome>(),
        //             )
        //         })
        //         .with_decoder(move |py, genotype| {
        //             if genotype.len() == 1 {
        //                 return PyAnyObject {
        //                     inner: PyList::new(
        //                         py,
        //                         genotype
        //                             .iter()
        //                             .flat_map(|chromo| {
        //                                 chromo
        //                                     .iter()
        //                                     .map(|gene| call_creator(py, gene).unwrap().inner)
        //                             })
        //                             .collect::<Vec<_>>(),
        //                     )
        //                     .unwrap()
        //                     .unbind()
        //                     .into_any(),
        //                 };
        //             }

        //             return PyAnyObject {
        //                 inner: PyList::new(
        //                     py,
        //                     genotype
        //                         .iter()
        //                         .map(|chromo| {
        //                             chromo
        //                                 .iter()
        //                                 .map(|gene| call_creator(py, gene).unwrap().inner)
        //                                 .collect::<Vec<_>>()
        //                         })
        //                         .collect::<Vec<_>>(),
        //                 )
        //                 .unwrap()
        //                 .unbind()
        //                 .into_any(),
        //             };
        //         }),
        // })
    }

    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(PyGenotype::from(self.codec.encode()))
    }

    pub fn decode_py<'py>(&self, py: Python<'py>, genotype: PyGenotype) -> PyResult<Py<PyAny>> {
        Ok(self.codec.decode_with_py(py, &genotype.into()).inner)
    }
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyFieldSpec {
    // pub field: Field,
    // pub init_range: Option<(f64, f64)>,
    // pub bounds: Option<(f64, f64)>,
    // pub chars: Option<Vec<char>>,
    // pub choices: Option<Vec<AnyValue<'static>>>,
    pub spec: FieldSpec,
}

#[pymethods]
impl PyFieldSpec {
    #[new]
    pub fn new(
        field: Wrap<Field>,
        init_range: Option<(f64, f64)>,
        bounds: Option<(f64, f64)>,
        chars: Option<Vec<char>>,
        choices: Option<Vec<Wrap<AnyValue<'_>>>>,
    ) -> Self {
        PyFieldSpec {
            spec: FieldSpec::Scalar {
                field: field.0,
                init_range: init_range.map(|(s, e)| s..e),
                bounds: bounds.map(|(s, e)| s..e),
                chars: chars.map(Arc::from),
                choices: choices.map(|c| c.into_iter().map(|w| w.0.into_static()).collect()),
            },
        }
    }

    /// Fixed-length list
    #[staticmethod]
    pub fn list(len: usize, inner: PyFieldSpec) -> Self {
        PyFieldSpec {
            spec: FieldSpec::List {
                len,
                inner: Box::new(inner.spec),
            },
        }
    }

    /// Struct made of fields
    #[staticmethod]
    pub fn struct_(fields: Vec<PyFieldSpec>) -> Self {
        PyFieldSpec {
            spec: FieldSpec::Struct {
                fields: fields.into_iter().map(|f| f.spec).collect(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum ScalarKind {
    Float,
    Int,
    Bool,
    CharSet(Arc<[char]>),              // index gene
    Choices(Arc<[AnyValue<'static>]>), // index gene
}

#[derive(Clone, Debug)]
pub enum LeafKind {
    Scalar { kind: ScalarKind, field: Field },
    List(Vec<LeafKind>), // fixed len list
}

#[derive(Clone, Debug)]
pub struct LeafPlan {
    pub path: Arc<[Arc<str>]>,
    pub kind: LeafKind,
    pub offset: usize,
    pub width: usize,
}

#[derive(Clone, Debug)]
pub struct MixedPlan {
    pub leaves: Vec<LeafPlan>,
    pub total_genes: usize,
}

#[derive(Clone, Debug)]
pub enum FieldSpec {
    Scalar {
        field: Field,
        init_range: Option<Range<f64>>,
        bounds: Option<Range<f64>>,
        chars: Option<Arc<[char]>>,
        choices: Option<Vec<AnyValue<'static>>>,
    },
    List {
        len: usize,
        inner: Box<FieldSpec>,
    },
    Struct {
        fields: Vec<FieldSpec>,
    },
}

fn compile_plan(spec: &FieldSpec) -> MixedPlan {
    let mut leaves = vec![];
    let mut offset = 0usize;
    let mut path: Vec<Arc<str>> = vec![];

    fn width_of(spec: &FieldSpec) -> usize {
        match spec {
            FieldSpec::Scalar { .. } => 1,
            FieldSpec::List { len, inner } => len * width_of(inner),
            FieldSpec::Struct { fields } => fields.iter().map(width_of).sum(),
        }
    }

    fn walk(
        spec: &FieldSpec,
        path: &mut Vec<Arc<str>>,
        leaves: &mut Vec<LeafPlan>,
        offset: &mut usize,
    ) {
        match spec {
            FieldSpec::Struct { fields } => {
                for f in fields {
                    walk(f, path, leaves, offset);
                }
            }
            FieldSpec::List { len, inner } => {
                // fixed len list: just repeat inner len times
                path.push(Arc::<str>::from(path.len().to_string()));
                for _ in 0..*len {
                    walk(inner, path, leaves, offset);
                }
                path.pop();
            }
            FieldSpec::Scalar {
                field,
                chars,
                choices,
                ..
            } => {
                let kind = if let Some(cs) = chars {
                    ScalarKind::CharSet(cs.clone())
                } else if let Some(ch) = choices {
                    ScalarKind::Choices(ch.clone().into())
                } else {
                    match field.dtype() {
                        DataType::Float32 | DataType::Float64 => ScalarKind::Float,
                        DataType::Int8
                        | DataType::Int16
                        | DataType::Int32
                        | DataType::Int64
                        | DataType::UInt8
                        | DataType::UInt16
                        | DataType::UInt32
                        | DataType::UInt64 => ScalarKind::Int,
                        DataType::Boolean => ScalarKind::Bool,
                        _ => panic!("Unsupported scalar type in FieldSpec"),
                    }
                };

                let w = 1;
                leaves.push(LeafPlan {
                    path: path.clone().into(),
                    kind: LeafKind::Scalar {
                        kind,
                        field: field.clone(),
                    },
                    offset: *offset,
                    width: w,
                });
                *offset += w;
            }
        }
    }

    walk(spec, &mut path, &mut leaves, &mut offset);

    MixedPlan {
        leaves,
        total_genes: offset,
    }
}

fn gene_for_scalar(spec: &FieldSpec) -> FloatGene<f64> {
    match spec {
        FieldSpec::Scalar {
            init_range,
            bounds,
            chars,
            choices,
            field,
        } => {
            // Choose ranges:
            if chars.is_some() {
                // index in [0, n-1]
                let n = chars.as_ref().unwrap().len().max(1) as f64;
                return FloatGene::from(0.0..(n - 1.0));
            }
            if let Some(ch) = choices {
                let n = ch.len().max(1) as f64;
                return FloatGene::from(0.0..(n - 1.0));
            }

            let (min, max) = match field.dtype().primitive_bounds() {
                Some((min, max)) => (
                    min.extract::<f64>().unwrap_or(0.0),
                    max.extract::<f64>().unwrap_or(1.0),
                ),
                None => (0.0, 1.0),
            };

            let init_range = init_range.as_ref().unwrap_or(&(min..max)).clone();
            let bounds = bounds.as_ref().unwrap_or(&init_range).clone();

            let clamped_init_range = init_range.start.max(min)..init_range.end.min(max);
            let clamped_bounds = bounds.start.max(min)..bounds.end.min(max);

            FloatGene::from((clamped_init_range, clamped_bounds))
        }
        _ => FloatGene::from((-1.0..1.0, -1.0..1.0)),
    }
}

fn encode_mixed(spec: &FieldSpec) -> FloatChromosome<f64> {
    fn emit(spec: &FieldSpec, out: &mut Vec<FloatGene<f64>>) {
        match spec {
            FieldSpec::Struct { fields } => fields.iter().for_each(|f| emit(f, out)),
            FieldSpec::List { len, inner } => {
                for _ in 0..*len {
                    emit(inner, out);
                }
            }
            FieldSpec::Scalar { .. } => out.push(gene_for_scalar(spec)),
        }
    }

    let mut genes = vec![];
    emit(spec, &mut genes);
    FloatChromosome::new(genes)
}

fn decode_plan(
    spec: &FieldSpec,
    chromo: &[FloatGene<f64>],
    offset: &mut usize,
) -> AnyValue<'static> {
    match spec {
        FieldSpec::Struct { fields } => AnyValue::Struct(
            fields
                .iter()
                .map(|f| {
                    let value = decode_plan(f, chromo, offset);
                    (
                        match f {
                            FieldSpec::Scalar { field, .. } => field.clone(),
                            FieldSpec::List { inner, .. } => match **inner {
                                FieldSpec::Scalar { ref field, .. } => field.clone(),
                                _ => panic!("Nested non-scalar in list is not supported"),
                            },
                            FieldSpec::Struct { .. } => panic!("Nested struct is not supported"),
                        },
                        value,
                    )
                })
                .collect(),
        ),
        FieldSpec::List { len, inner } => {
            let mut items = Vec::with_capacity(*len);
            for _ in 0..*len {
                items.push(decode_plan(inner, chromo, offset));
            }
            AnyValue::Vector(Box::new(items))
        }
        FieldSpec::Scalar {
            chars,
            choices,
            field,
            ..
        } => {
            let gene = &chromo[*offset];
            *offset += 1;

            if let Some(cs) = chars {
                let idx = gene.allele().round() as usize;
                return cs
                    .get(idx)
                    .cloned()
                    .map(AnyValue::Char)
                    .unwrap_or(AnyValue::Null);
            }

            if let Some(ch) = choices {
                let idx = gene.allele().round() as usize;
                return ch.get(idx).cloned().unwrap_or(AnyValue::Null);
            }

            match field.dtype() {
                DataType::Float32 => {
                    AnyValue::Float32(gene.allele().extract::<f32>().unwrap_or_default())
                }
                DataType::Float64 => AnyValue::Float64(*gene.allele()),
                DataType::Int8 => AnyValue::Int8(gene.allele().extract::<i8>().unwrap()),
                DataType::Int16 => AnyValue::Int16(gene.allele().extract::<i16>().unwrap()),
                DataType::Int32 => AnyValue::Int32(gene.allele().extract::<i32>().unwrap()),
                DataType::Int64 => AnyValue::Int64(gene.allele().extract::<i64>().unwrap()),
                DataType::UInt8 => AnyValue::UInt8(gene.allele().extract::<u8>().unwrap()),
                DataType::UInt16 => AnyValue::UInt16(gene.allele().extract::<u16>().unwrap()),
                DataType::UInt32 => AnyValue::UInt32(gene.allele().extract::<u32>().unwrap()),
                DataType::UInt64 => AnyValue::UInt64(gene.allele().extract::<u64>().unwrap()),
                DataType::Boolean => {
                    AnyValue::Bool(gene.allele().extract::<f64>().unwrap_or_default() > 0.5)
                }
                _ => AnyValue::Null,
            }
        }
    }
}

pub struct FlatSchemaCodec {
    plan: MixedPlan,
    spec: FieldSpec,
}

impl FlatSchemaCodec {
    pub fn new(spec: FieldSpec) -> Self {
        let plan = compile_plan(&spec);
        FlatSchemaCodec { plan, spec }
    }

    pub fn plan(&self) -> &MixedPlan {
        &self.plan
    }
}

impl Codec<FloatChromosome<f64>, Vec<AnyValue<'static>>> for FlatSchemaCodec {
    fn encode(&self) -> Genotype<FloatChromosome<f64>> {
        Genotype::from(encode_mixed(&self.spec))
    }

    fn decode(&self, genotype: &Genotype<FloatChromosome<f64>>) -> Vec<AnyValue<'static>> {
        let mut result = Vec::new();
        for leaf in &self.plan.leaves {
            let value = decode_plan(&self.spec, &genotype[0].as_slice(), &mut 0);
            result.push(value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fld(name: &str, dt: DataType) -> Field {
        Field::from((name, dt))
    }

    #[test]
    fn compile_and_encode_gene_count_matches() {
        // lr: float, depth: int, use_bias: bool, layers: list[int; 4]
        let spec = FieldSpec::Struct {
            fields: vec![
                FieldSpec::Scalar {
                    field: fld("lr", DataType::Float64),
                    init_range: Some(1e-5..1e-1),
                    bounds: Some(1e-5..1e-1),
                    chars: None,
                    choices: None,
                },
                FieldSpec::Scalar {
                    field: fld("depth", DataType::Int32),
                    init_range: Some(1.0..12.0),
                    bounds: Some(1.0..12.0),
                    chars: None,
                    choices: None,
                },
                FieldSpec::Scalar {
                    field: fld("use_bias", DataType::Boolean),
                    init_range: Some(0.0..1.0),
                    bounds: Some(0.0..1.0),
                    chars: None,
                    choices: None,
                },
                FieldSpec::List {
                    len: 4,
                    inner: Box::new(FieldSpec::Scalar {
                        field: fld("layers", DataType::Int32),
                        init_range: Some(8.0..256.0),
                        bounds: Some(8.0..256.0),
                        chars: None,
                        choices: None,
                    }),
                },
                FieldSpec::Scalar {
                    field: fld("activation", DataType::String),
                    init_range: None,
                    bounds: None,
                    chars: None,
                    choices: Some(vec![
                        AnyValue::StrOwned("relu".into()),
                        AnyValue::StrOwned("tanh".into()),
                        AnyValue::StrOwned("sigmoid".into()),
                    ]),
                },
            ],
        };

        let codec = FlatSchemaCodec::new(spec);
        assert_eq!(codec.plan().total_genes, 1 + 1 + 1 + 4 + 1);

        let gt = codec.encode();
        let chrom = gt.get(0).unwrap();
        assert_eq!(chrom.len(), codec.plan().total_genes);
    }

    #[test]
    fn decode_produces_struct_with_vector_for_repeated_field() {
        let spec = FieldSpec::Struct {
            fields: vec![
                FieldSpec::Scalar {
                    field: fld("lr", DataType::Float64),
                    init_range: Some(0.0..1.0),
                    bounds: Some(0.0..1.0),
                    chars: None,
                    choices: None,
                },
                FieldSpec::List {
                    len: 3,
                    inner: Box::new(FieldSpec::Scalar {
                        field: fld("layers", DataType::Int32),
                        init_range: Some(0.0..10.0),
                        bounds: Some(0.0..10.0),
                        chars: None,
                        choices: None,
                    }),
                },
            ],
        };

        let codec = FlatSchemaCodec::new(spec);
        let gt = codec.encode();
        let decoded = codec.decode(&gt)[0].clone();

        match decoded {
            AnyValue::Struct(fields) => {
                // We should have "lr" and "layers"
                assert!(fields.iter().any(|(f, _)| f.name() == "lr"));
                let layers = fields.iter().find(|(f, _)| f.name() == "layers").unwrap();

                match &layers.1 {
                    AnyValue::Vector(v) => assert_eq!(v.len(), 3),
                    other => panic!("expected layers to be Vector, got {other:?}"),
                }
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }
}
