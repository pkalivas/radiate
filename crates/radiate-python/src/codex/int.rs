use crate::{AnyValue, DataType, Field};
use pyo3::{pyclass, pymethods};
use radiate::{Chromosome, FnCodex, Gene, IntChromosome};

#[pyclass]
#[derive(Clone)]
pub struct PyIntCodex {
    pub codex: FnCodex<IntChromosome<i32>, AnyValue<'static>>,
}

unsafe impl Send for PyIntCodex {}
unsafe impl Sync for PyIntCodex {}

#[pymethods]
impl PyIntCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn new(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
    ) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let val_range = value_range.unwrap_or((0, 1));
        let bound_range = bound_range.unwrap_or(val_range);
        let val_range = val_range.0..val_range.1;
        let bound_range = bound_range.0..bound_range.1;
        PyIntCodex {
            codex: FnCodex::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| {
                            IntChromosome::from((*len, val_range.clone(), bound_range.clone()))
                        })
                        .collect::<Vec<IntChromosome<i32>>>()
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
                                std::any::type_name::<Vec<i32>>().to_string(),
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
                            std::any::type_name::<Vec<Vec<i32>>>().to_string(),
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
