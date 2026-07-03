// #[cfg(feature = "serde")]
// use serde::{Deserialize, Serialize};
use std::ops::Range;

use crate::{BoundedGene, RangeLookup};

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
