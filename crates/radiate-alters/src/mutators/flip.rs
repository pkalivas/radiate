use radiate_core::{
    AlterContext, BitGene, ContiguousChromosome, Expr, Mutate, RateSet, random_provider,
};

const GEOMETRIC_FLIP_THRESHOLD: f32 = 0.4;

/// A bit-flip mutator that flips genes in a chromosome with a given probability.
///
/// For small mutation probabilities, the mutator uses a geometric distribution
/// to efficiently skip genes that will not be mutated. For larger probabilities,
/// it evaluates each gene independently and flips it with probability `p`.
///
/// The choice of strategy is based on the mutation probability and is intended
/// to minimize the cost of mutation across different probability ranges.
///
/// The following benchmark compares the two strategies on a relatively large
/// chromosome (1000 genes). Values are reported in ns per call:
///
/// | Probability `p` | Geometric | Per-gene |
/// |----------------:|----------:|---------:|
/// | 0.0002          |        19 |     1245 |
/// | 0.01            |       109 |     1257 |
/// | 0.1             |       699 |     1696 |
/// | 0.4             |      2691 |     3435 |
/// | 0.5             |      3357 |     3782 |
/// | 0.7             |      4749 |     2692 |
///
/// The geometric strategy is great for low mutation
/// probabilities, where most genes can be skipped without generating a random
/// value for each gene. As the probability increases, the per-gene strategy
/// becomes more competitive and eventually faster.
#[derive(Debug, Clone)]
pub struct BitFlipMutator {
    rate: Expr,
}

impl BitFlipMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into() }
    }
}

impl<C> Mutate<C> for BitFlipMutator
where
    C: ContiguousChromosome<Gene = BitGene>,
{
    fn rates(&self) -> radiate_core::RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let p = ctx.rate();
        debug_assert!(p.is_finite());

        if p <= 0.0 {
            return 0;
        } else if p >= 1.0 {
            // just invert all genes
            for gene in chromosome.as_mut_slice() {
                gene.flip();
            }

            return chromosome.len();
        }

        if p <= GEOMETRIC_FLIP_THRESHOLD {
            geometric_flip(chromosome.as_mut_slice(), p as f64)
        } else {
            random_flip(chromosome.as_mut_slice(), p as f64)
        }
    }
}

#[inline]
fn geometric_flip(genes: &mut [BitGene], p: f64) -> usize {
    let len = genes.len();
    if len == 0 || p <= 0.0 {
        return 0;
    }

    let ln_q = (-p).ln_1p();
    let mut flips = 0;
    random_provider::with_rng(|rng| {
        let mut i = 0;
        while i < len {
            let u = rng.random::<f64>();
            let gap = ((1.0 - u).ln() / ln_q).floor();

            if gap >= (len - i) as f64 {
                break;
            }

            i += gap as usize;
            genes[i].flip();
            flips += 1;
            i += 1;
        }
    });

    flips
}

#[inline]
fn random_flip(genes: &mut [BitGene], p: f64) -> usize {
    if p <= 0.0 {
        return 0;
    }

    let mut flips = 0;
    random_provider::with_rng(|rng| {
        for gene in genes.iter_mut() {
            if rng.bool(p) {
                gene.flip();
                flips += 1;
            }
        }
    });

    flips
}
