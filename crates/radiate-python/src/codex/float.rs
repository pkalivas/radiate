use crate::{AnyValue, DataType, Field};
use pyo3::{pyclass, pymethods};
use radiate::{Chromosome, FloatChromosome, FnCodex, Gene};

#[pyclass]
#[derive(Clone)]
pub struct PyFloatCodex {
    pub codex: FnCodex<FloatChromosome, AnyValue<'static>>,
}

unsafe impl Send for PyFloatCodex {}
unsafe impl Sync for PyFloatCodex {}

#[pymethods]
impl PyFloatCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn new(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
    ) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let val_range = value_range.unwrap_or((0.0, 1.0));
        let bound_range = bound_range.unwrap_or(val_range);
        let val_range = val_range.0..val_range.1;
        let bound_range = bound_range.0..bound_range.1;
        PyFloatCodex {
            codex: FnCodex::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| {
                            FloatChromosome::from((*len, val_range.clone(), bound_range.clone()))
                        })
                        .collect::<Vec<FloatChromosome>>()
                        .into()
                })
                .with_decoder(|geno| {
                    let mut list = Vec::new();
                    for chromo in geno.iter() {
                        let mut genes = Vec::new();
                        for gene in chromo.iter() {
                            genes.push(AnyValue::from(*gene.allele()));
                        }
                        list.push(AnyValue::VecOwned(Box::new((
                            genes,
                            Field::new(
                                std::any::type_name::<Vec<f32>>().to_string(),
                                DataType::List(Box::new(Field::new(
                                    "item".to_string(),
                                    DataType::Null,
                                ))),
                            ),
                        ))));
                    }

                    AnyValue::VecOwned(Box::new((
                        list,
                        Field::new(
                            std::any::type_name::<Vec<Vec<f32>>>().to_string(),
                            DataType::List(Box::new(Field::new(
                                "item".to_string(),
                                DataType::Null,
                            ))),
                        ),
                    )))
                }),
        }
    }
}
