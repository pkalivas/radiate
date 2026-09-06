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
pub use gene::{BoundedGene, Gene, NumericGene, Valid};
pub use int::{IntChromosome, IntGene};
use num_traits::NumCast;
pub use permutation::{PermutationChromosome, PermutationGene};
use radiate_utils::Primitive;

pub trait NumericAllele: Primitive {
    fn extract<T: NumCast>(self) -> Option<T> {
        T::from(self)
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

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BoundedSequence<T> {
    data: Vec<T>,
    init_range: RangeLookup<T>,
    bounds: RangeLookup<T>,
}

impl<G, T> From<Vec<G>> for BoundedSequence<T>
where
    G: BoundedGene<Allele = T>,
    T: Clone + PartialEq,
{
    fn from(data: Vec<G>) -> Self {
        let init_range = build_init_range_loookup(&data);
        let bounds = build_bounds_lookup(&data);
        Self {
            data: data.into_iter().map(|gene| gene.allele().clone()).collect(),
            init_range,
            bounds,
        }
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
        let min_allele = (*gene.init_min()).clone();
        let max_allele = (*gene.init_max()).clone();

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
        let (min, max) = gene.bound_range();

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

// /// A read handle to a single gene's data, possibly synthesized rather than
// /// borrowed directly — lets a Chromosome expose gene-shaped access regardless
// /// of whether it stores real Gene values or decomposed (SoA) fields.
// pub trait GeneView<'a, G: Gene> {
//     fn allele(&self) -> &G::Allele;
// }

// pub trait GeneViewMut<'a, G: Gene>: GeneView<'a, G> {
//     fn allele_mut(&mut self) -> &mut G::Allele;
//     fn set_allele(&mut self, allele: G::Allele);
// }

// pub trait BoundedGeneView<'a, G: BoundedGene>: GeneView<'a, G> {
//     fn bound_min(&self) -> &G::Allele;
//     fn bound_max(&self) -> &G::Allele;
// }

// impl<'a, G: Gene> GeneView<'a, G> for &'a G {
//     fn allele(&self) -> &G::Allele {
//         Gene::allele(*self)
//     }
// }

// impl<'a, G: Gene> GeneView<'a, G> for &'a mut G {
//     fn allele(&self) -> &G::Allele {
//         Gene::allele(*self)
//     }
// }
// impl<'a, G: Gene> GeneViewMut<'a, G> for &'a mut G {
//     fn allele_mut(&mut self) -> &mut G::Allele {
//         Gene::allele_mut(*self)
//     }
//     fn set_allele(&mut self, allele: G::Allele) {
//         Gene::set_allele(*self, allele)
//     }
// }
// impl<'a, G: BoundedGene> BoundedGeneView<'a, G> for &'a G {
//     fn bound_min(&self) -> &G::Allele {
//         BoundedGene::bound_min(*self)
//     }
//     fn bound_max(&self) -> &G::Allele {
//         BoundedGene::bound_max(*self)
//     }
// }

// pub trait ChromosomeView: Chromosome {
//     type View<'b>: GeneView<'b, Self::Gene>
//     where
//         Self: 'b,
//         Self::Gene: Gene;

//     fn view(&self, index: usize) -> Option<Self::View<'_>>;
// }

// pub trait ChromosomeViewMut: ChromosomeView {
//     type ViewMut<'b>: GeneViewMut<'b, Self::Gene>
//     where
//         Self: 'b,
//         Self::Gene: Gene;

//     fn view_mut(&mut self, index: usize) -> Option<Self::ViewMut<'_>>;
// }

// pub struct FloatGeneView<'a, F> {
//     allele: &'a F,
//     bounds: &'a Range<F>,
// }

// impl<'a, F: Float> GeneView<'a, FloatGene<F>> for FloatGeneView<'a, F> {
//     fn allele(&self) -> &F {
//         self.allele
//     }
// }
// impl<'a, F: Float> BoundedGeneView<'a, FloatGene<F>> for FloatGeneView<'a, F> {
//     fn bound_min(&self) -> &F {
//         &self.bounds.start
//     }
//     fn bound_max(&self) -> &F {
//         &self.bounds.end
//     }
// }

// impl<F: Float> ChromosomeView for FloatChromosome<F> {
//     type View<'b>
//         = FloatGeneView<'b, F>
//     where
//         Self: 'b;

//     fn view(&self, index: usize) -> Option<Self::View<'_>> {
//         self.get(index).map(|gene| FloatGeneView {
//             allele: &gene.allele,
//             bounds: &(gene.bounds),
//         })
//     }
// }

// impl<F: Float> ChromosomeViewMut for FloatChromosome<F> {
//     type ViewMut<'b>
//         = FloatGeneViewMut<'b, F>
//     where
//         Self: 'b;

//     fn view_mut(&mut self, index: usize) -> Option<Self::ViewMut<'_>> {
//         self.get_mut(index).map(|gene| FloatGeneViewMut {
//             allele: &mut gene.allele,
//             bounds: &mut gene.bounds,
//         })
//     }
// }

// #[derive(Clone, PartialEq, Default)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct DenseGeneStore<G: Gene> {
//     genes: Vec<G>,
// }

// #[derive(Clone, PartialEq, Default)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct SharedBoundedGeneStore<A> {
//     alleles: Vec<A>,
//     bounds: (A, A),
// }

// pub trait GeneStore<G: Gene> {
//     type View<'a>: GeneView<'a, G>
//     where
//         Self: 'a,
//         G: Gene;
//     type ViewMut<'a>: GeneViewMut<'a, G>
//     where
//         Self: 'a,
//         G: Gene;
//     fn get(&self, index: usize) -> Option<Self::View<'_>>;
//     fn get_mut(&mut self, index: usize) -> Option<Self::ViewMut<'_>>;
//     fn len(&self) -> usize;
//     fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
//     fn iter(&self) -> impl Iterator<Item = Self::View<'_>> {
//         (0..self.len()).filter_map(|i| self.get(i))
//     }
//     fn iter_mut(&mut self) -> impl Iterator<Item = Self::ViewMut<'_>>;
// }

// impl<F: Float> GeneStore<FloatGene<F>> for DenseGeneStore<FloatGene<F>> {
//     type View<'a>
//         = FloatGeneView<'a, F>
//     where
//         Self: 'a,
//         F: Float;

//     type ViewMut<'a>
//         = FloatGeneViewMut<'a, F>
//     where
//         Self: 'a,
//         F: Float;

//     fn get(&self, index: usize) -> Option<Self::View<'_>> {
//         self.genes.get(index).map(|gene| FloatGeneView {
//             allele: &gene.allele,
//             bounds: &gene.bounds,
//         })
//     }

//     fn get_mut(&mut self, index: usize) -> Option<Self::ViewMut<'_>> {
//         self.genes.get_mut(index).map(|gene| FloatGeneViewMut {
//             allele: &mut gene.allele,
//             bounds: &mut gene.bounds,
//         })
//     }

//     fn len(&self) -> usize {
//         self.genes.len()
//     }

//     fn iter(&self) -> impl Iterator<Item = Self::View<'_>> {
//         self.genes.iter().map(|gene| FloatGeneView {
//             allele: &gene.allele,
//             bounds: &gene.bounds,
//         })
//     }

//     fn iter_mut(&mut self) -> impl Iterator<Item = Self::ViewMut<'_>> {
//         self.genes.iter_mut().map(|gene| FloatGeneViewMut {
//             allele: &mut gene.allele,
//             bounds: &mut gene.bounds,
//         })
//     }
// }
