use crate::{
    PyEngineInput,
    bindings::converters::{
        alters::{
            bit_alterers, char_alterers, float_alterers, graph_alterers, int_alterers,
            packed_bit_alterers, perm_alterers, tree_alterers,
        },
        diversity::{
            bit_distances, char_distances, float_distances, graph_distances, int_distances,
            packed_bit_distances, perm_distances, tree_distances,
        },
    },
};
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, GraphChromosome, IntChromosome, Op,
    PackedBitChromosome, PermutationChromosome, RadiateResult, TreeChromosome,
};
use radiate_error::radiate_bail;

mod alters;
mod diversity;
mod executors;
mod filters;
mod limits;
mod selectors;

/// Builds a Radiate type from a [`PyEngineInput`] - the configuration Python hands over
/// for each engine component. Implemented at every level: concrete components
/// (`UniformCrossover`), and the per-chromosome types the builder asks for
/// (`Alterer<C>`, `Box<dyn Select<C>>`, `Limit`, ...), which dispatch by name through a
/// [`Registry`].
pub trait FromPyEngineInput: Sized {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self>;
}

pub struct Registry<O: 'static> {
    pub(crate) kind: &'static str,
    pub(crate) entries: &'static [(&'static str, fn(&PyEngineInput) -> RadiateResult<O>)],
}

impl<O> Registry<O> {
    pub fn convert(&self, input: &PyEngineInput) -> RadiateResult<O> {
        let name = input.component();
        match self.entries.iter().find(|(n, _)| *n == name) {
            Some((_, build)) => build(input),
            None => {
                let valid = self
                    .entries
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ");
                radiate_bail!(Builder: "Invalid {} '{}'. Valid: [{}]", self.kind, name, valid)
            }
        }
    }
}

#[macro_export]
macro_rules! registry {
    ($kind:literal, $wrap:expr; { $($name:path => $ty:ty),* $(,)? }) => {
        Registry {
            kind: $kind,
            entries: &[$(($name, |input| <$ty as FromPyEngineInput>::from_input(input).map($wrap))),*],
        }
    };
}

#[macro_export]
macro_rules! impl_from_input {
    ($chrom:ty, alterers: $alters:expr, distances: $dists:expr) => {
        impl FromPyEngineInput for radiate::Alterer<$chrom> {
            fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
                input.expect_type(crate::PyEngineInputType::Alterer)?;
                $alters.convert(input)
            }
        }

        impl FromPyEngineInput
            for Box<dyn radiate::distance::Distance<radiate::Phenotype<$chrom>>>
        {
            fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
                input.expect_type(crate::PyEngineInputType::Diversity)?;
                $dists.convert(input)
            }
        }
    };
}

impl_from_input!(IntChromosome<u8>, alterers: int_alterers::<u8>(), distances: int_distances::<u8>());
impl_from_input!(IntChromosome<u16>, alterers: int_alterers::<u16>(), distances: int_distances::<u16>());
impl_from_input!(IntChromosome<u32>, alterers: int_alterers::<u32>(), distances: int_distances::<u32>());
impl_from_input!(IntChromosome<u64>, alterers: int_alterers::<u64>(), distances: int_distances::<u64>());

impl_from_input!(IntChromosome<i8>, alterers: int_alterers::<i8>(), distances: int_distances::<i8>());
impl_from_input!(IntChromosome<i16>, alterers: int_alterers::<i16>(), distances: int_distances::<i16>());
impl_from_input!(IntChromosome<i32>, alterers: int_alterers::<i32>(), distances: int_distances::<i32>());
impl_from_input!(IntChromosome<i64>, alterers: int_alterers::<i64>(), distances: int_distances::<i64>());

impl_from_input!(FloatChromosome<f32>, alterers: float_alterers::<f32>(), distances: float_distances::<f32>());
impl_from_input!(FloatChromosome<f64>, alterers: float_alterers::<f64>(), distances: float_distances::<f64>());

impl_from_input!(BitChromosome, alterers: bit_alterers(), distances: bit_distances());
impl_from_input!(PackedBitChromosome, alterers: packed_bit_alterers(), distances: packed_bit_distances());
impl_from_input!(CharChromosome, alterers: char_alterers(), distances: char_distances());
impl_from_input!(PermutationChromosome<usize>, alterers: perm_alterers(), distances: perm_distances());

impl_from_input!(GraphChromosome<Op<f32>>, alterers: graph_alterers(), distances: graph_distances());
impl_from_input!(GraphChromosome<Op<f64>>, alterers: graph_alterers(), distances: graph_distances());

impl_from_input!(TreeChromosome<Op<f32>>, alterers: tree_alterers(), distances: tree_distances());
impl_from_input!(TreeChromosome<Op<f64>>, alterers: tree_alterers(), distances: tree_distances());
