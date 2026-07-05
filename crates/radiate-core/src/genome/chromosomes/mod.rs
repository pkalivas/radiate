pub mod bit;
pub mod char;
pub mod chromosome;
pub mod float;
pub mod gene;
pub mod int;
pub mod permutation;

pub use bit::{BitChromosome, BitGene};
pub use char::{CharChromosome, CharGene};
pub use chromosome::*;
pub use float::{FloatChromosome, FloatGene};
pub use gene::{ArithmeticGene, BoundedGene, Gene, NumericGene, Valid};
pub use int::{IntChromosome, IntGene};
use num_traits::NumCast;
pub use permutation::{PermutationChromosome, PermutationGene};
use radiate_utils::Primitive;

pub trait NumericAllele: Primitive {
    fn extract<T: NumCast>(self) -> Option<T> {
        T::from(self)
    }

    fn clamp(&mut self, min: &Self, max: &Self) {
        if *self < *min {
            *self = *min;
        } else if *self > *max {
            *self = *max;
        }
    }
}

macro_rules! impl_numeric_allele {
    ($($t:ty),*) => {
        $(
            impl NumericAllele for $t {}

        )*
    };
}

impl_numeric_allele!(f32, f64, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_valid {
    ($($t:ty),*) => {
        $(
            impl Valid for $t {
                #[inline]
                fn is_valid(&self) -> bool {
                    true
                }
            }
        )*
    };
}

impl_valid!(
    bool, char, String, isize, usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64,
    &str
);

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{ops::Range, sync::Arc};

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangeLookup<T> {
    bounds: Arc<[Range<T>]>,
    index_to_bounds_map: Arc<[usize]>,
}

impl<T> RangeLookup<T> {
    pub fn new(bounds: Vec<Range<T>>, index_to_bounds_map: Vec<usize>) -> Self {
        Self {
            bounds: Arc::from(bounds),
            index_to_bounds_map: Arc::from(index_to_bounds_map),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Range<T>> {
        self.index_to_bounds_map
            .get(index)
            .and_then(|&bounds_index| self.bounds.get(bounds_index))
    }
}

fn build_init_range_loookup<G>(genes: &[G]) -> RangeLookup<G::Allele>
where
    G: BoundedGene,
    G::Allele: Clone + PartialEq,
{
    let mut bounds: Vec<Range<G::Allele>> = Vec::new();
    let mut index_to_bounds_map = Vec::new();

    for gene in genes {
        let min_allele = (*gene.min()).clone();
        let max_allele = (*gene.max()).clone();

        let bounds_index = bounds
            .iter()
            .position(|r| r.start == min_allele && r.end == max_allele)
            .unwrap_or_else(|| {
                bounds.push(min_allele.clone()..max_allele.clone());
                bounds.len() - 1
            });

        index_to_bounds_map.push(bounds_index);
    }

    RangeLookup::new(bounds, index_to_bounds_map)
}

fn build_bounds_lookup<G>(genes: &[G]) -> RangeLookup<G::Allele>
where
    G: BoundedGene,
    G::Allele: Clone + PartialEq,
{
    let mut bounds: Vec<Range<G::Allele>> = Vec::new();
    let mut index_to_bounds_map = Vec::new();

    for gene in genes {
        let (min, max) = gene.bounds();

        let bounds_index = bounds
            .iter()
            .position(|r| r.start == *min && r.end == *max)
            .unwrap_or_else(|| {
                bounds.push(min.clone()..max.clone());
                bounds.len() - 1
            });

        index_to_bounds_map.push(bounds_index);
    }

    RangeLookup::new(bounds, index_to_bounds_map)
}

#[derive(Clone, PartialEq)]
pub struct BoundedFixedSequence<T, const N: usize> {
    data: [T; N],
    init_range: RangeLookup<T>,
    bounds: RangeLookup<T>,
}

impl<G, T, const N: usize> From<[G; N]> for BoundedFixedSequence<T, N>
where
    G: BoundedGene<Allele = T>,
    T: Clone + PartialEq,
{
    fn from(data: [G; N]) -> Self {
        let init_range = build_init_range_loookup(&data);
        let bounds = build_bounds_lookup(&data);
        Self {
            data: data.map(|gene| gene.allele().clone()),
            init_range,
            bounds,
        }
    }
}

impl<G, T, const N: usize> TryFrom<Vec<G>> for BoundedFixedSequence<T, N>
where
    G: BoundedGene<Allele = T>,
    T: Clone + PartialEq,
{
    type Error = Vec<G>;

    fn try_from(genes: Vec<G>) -> Result<Self, Self::Error> {
        let arr: [G; N] = genes.try_into()?;
        Ok(Self::from(arr))
    }
}
