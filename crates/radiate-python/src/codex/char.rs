use pyo3::{pyclass, pymethods};
use radiate::{CharChromosome, Chromosome, FnCodex, Gene};

use crate::{AnyValue, DataType, Field};

#[pyclass]
#[derive(Clone)]
pub struct PyCharCodex {
    pub codex: FnCodex<CharChromosome, AnyValue<'static>>,
}

unsafe impl Send for PyCharCodex {}
unsafe impl Sync for PyCharCodex {}

#[pymethods]
impl PyCharCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, char_set=None))]
    pub fn new(chromosome_lengths: Option<Vec<usize>>, char_set: Option<String>) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);

        PyCharCodex {
            codex: FnCodex::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| CharChromosome::from((*len, char_set.clone())))
                        .collect::<Vec<CharChromosome>>()
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
                                std::any::type_name::<Vec<char>>().to_string(),
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
                            std::any::type_name::<Vec<Vec<char>>>().to_string(),
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
