use std::{fmt::Debug, ops::Range};

use crate::BoundedGene;

use super::{Valid, gene::Gene};

/// The [Chromosome] is part of the genetic makeup of an individual.
/// It is a collection of [Gene] instances, it is essentially a
/// light wrapper around a Vec of [Gene]s. The [Chromosome] struct, however, has some additional
/// functionality and terminology that aligns with the biological concept of a chromosome
///
/// In traditional biological terms, a [Chromosome] is a long DNA molecule with part or all of the
/// genetic material of an organism. The [Chromosome] is the 'genetic' part of the individual that is
/// being evolved by the genetic algorithm.
///
/// We can think of a [Chromosome] as a Vec of structs which implement the [Gene] trait. For example,
/// if we have a [Chromosome] with 3 [Gene]s, it is represented as follows:
/// ```text
/// Chromosome: [Gene, Gene, Gene]
/// ```
pub trait Chromosome: Valid {
    type Gene: Gene;

    fn as_slice(&self) -> &[Self::Gene];
    fn as_mut_slice(&mut self) -> &mut [Self::Gene];

    fn get(&self, index: usize) -> Option<&Self::Gene> {
        self.as_slice().get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        self.as_mut_slice().get_mut(index)
    }

    fn set(&mut self, index: usize, gene: Self::Gene) {
        self.as_mut_slice()[index] = gene;
    }

    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> impl Iterator<Item = &Self::Gene> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Gene> {
        self.as_mut_slice().iter_mut()
    }

    fn zip_mut<'a>(
        &'a mut self,
        other: &'a mut Self,
    ) -> impl Iterator<Item = (&'a mut Self::Gene, &'a mut Self::Gene)> {
        self.iter_mut().zip(other.iter_mut())
    }
}

// #[cfg(feature = "serde")]
// use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangeLookup<T> {
    bounds: Arc<[Range<T>]>,
}

impl<T> RangeLookup<T> {
    pub fn new(bounds: Vec<Range<T>>) -> Self {
        Self {
            bounds: Arc::from(bounds),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Range<T>> {
        self.bounds.get(index)
    }
}

impl<T> Debug for RangeLookup<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RangeLookup")
            .field("bounds", &self.bounds)
            .finish()
    }
}

fn build_init_range_loookup<G>(genes: &[G]) -> RangeLookup<G::Allele>
where
    G: BoundedGene,
    G::Allele: Clone + PartialEq,
{
    let mut bounds: Vec<Range<G::Allele>> = Vec::new();

    let mut current_index = 0;
    let mut current_range: Option<Range<G::Allele>> = None;

    while current_index < genes.len() {
        let gene = &genes[current_index];
        let (min, max) = gene.bounds();

        match &mut current_range {
            Some(range) => {
                if range.start == *min && range.end == *max {
                    // Same range, continue
                } else {
                    // Different range, push the current range and start a new one
                    bounds.push(range.clone());
                    current_range = Some(min.clone()..max.clone());
                }
            }
            None => {
                // First range
                current_range = Some(min.clone()..max.clone());
            }
        }

        current_index += 1;
    }

    if let Some(range) = current_range {
        bounds.push(range);
    }

    RangeLookup::new(bounds)
}

fn build_bounds_lookup<G>(genes: &[G]) -> RangeLookup<G::Allele>
where
    G: BoundedGene,
    G::Allele: Clone + PartialEq,
{
    let mut bounds: Vec<Range<G::Allele>> = Vec::new();

    let mut current_index = 0;
    let mut current_range: Option<Range<G::Allele>> = None;

    while current_index < genes.len() {
        let gene = &genes[current_index];
        let (min, max) = gene.bounds();

        match &mut current_range {
            Some(range) => {
                if range.start == *min && range.end == *max {
                    // Same range, continue
                } else {
                    // Different range, push the current range and start a new one
                    bounds.push(range.clone());
                    current_range = Some(min.clone()..max.clone());
                }
            }
            None => {
                // First range
                current_range = Some(min.clone()..max.clone());
            }
        }

        current_index += 1;
    }

    if let Some(range) = current_range {
        bounds.push(range);
    }

    RangeLookup::new(bounds)
}

#[derive(Clone, PartialEq)]
pub struct BoundedFixedSequence<T> {
    data: Vec<T>,
    init_range: RangeLookup<T>,
    bounds: RangeLookup<T>,
}

impl<G, T> From<Vec<G>> for BoundedFixedSequence<T>
where
    G: BoundedGene<Allele = T>,
    T: Clone + PartialEq,
{
    fn from(genes: Vec<G>) -> Self {
        let init_range = build_init_range_loookup(&genes);
        let bounds = build_bounds_lookup(&genes);
        Self {
            data: genes
                .into_iter()
                .map(|gene| gene.allele().clone())
                .collect(),
            init_range,
            bounds,
        }
    }
}

impl<T> Debug for BoundedFixedSequence<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundedFixedSequence")
            .field("data", &self.data)
            .field("init_range", &self.init_range)
            .field("bounds", &self.bounds)
            .finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{FloatChromosome, FloatGene};

    #[test]
    fn test_build_init_range_lookup() {
        let float_chromosome = FloatChromosome::from(vec![
            FloatGene::new(0.0, 0.0..1.0, 0.0..1.0),
            FloatGene::new(0.5, 0.0..1.0, 0.0..1.0),
            FloatGene::new(0.2, 0.0..1.0, 0.0..1.0),
            FloatGene::new(0.3, 0.0..2.0, -2.0..1.0),
            FloatGene::new(0.4, 0.0..1.0, 0.0..1.0),
            FloatGene::new(0.5, -4.0..1.0, -5.0..1.0),
            FloatGene::new(0.6, 0.0..1.0, -10.0..10.0),
            FloatGene::new(0.7, 0.0..1.0, 0.0..1.0),
        ])
        .into_iter()
        .collect::<Vec<_>>();

        let bounded_sequence = BoundedFixedSequence::from(float_chromosome);

        println!("{:#?}", bounded_sequence);
    }
}
