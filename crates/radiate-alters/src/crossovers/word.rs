use radiate_core::{
    AlterContext, BitWordGene, ContiguousChromosome, Crossover, Expr, Gene, PackedBitChromosome,
    RateSet, random_provider,
};

/// Uniform crossover at bit granularity for [`PackedBitChromosome`].
///
/// Each bit position is swapped between the two parents with probability `rate`.
/// The generic [`UniformCrossover`](crate::UniformCrossover) swaps whole 64-bit words
/// on a packed chromosome. This one swaps individual bits.
///
/// The rate is used twice, the same as `UniformCrossover`: it decides which pairs are
/// crossed, and then the per-bit swap probability within a crossed pair. Selected bits
/// are gathered into one mask per word and swapped with a single masked XOR.
///
/// The returned count is the number of bits selected for swapping.
#[derive(Debug, Clone)]
pub struct PackedBitCrossover {
    rate: Expr,
}

impl PackedBitCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into() }
    }
}

impl Crossover<PackedBitChromosome> for PackedBitCrossover {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    fn cross_chromosomes(
        &self,
        one: &mut PackedBitChromosome,
        two: &mut PackedBitChromosome,
        ctx: &mut AlterContext,
    ) -> usize {
        let num_bits = one.num_bits().min(two.num_bits());
        if num_bits == 0 {
            return 0;
        }

        let (a, b) = (one.as_mut_slice(), two.as_mut_slice());
        let mut swap = |w: usize, mask: u64| swap_masked(a, b, w, mask);

        // Positions arrive in ascending order: build one mask per word and apply it
        // when the next position moves to a new word.
        let (mut cur, mut mask, mut swapped) = (0, 0u64, 0);
        random_provider::with_rng(|rng| {
            rng.bernoulli_indices(ctx.rate(), num_bits, |k| {
                if k >> 6 != cur {
                    swap(cur, mask);
                    (cur, mask) = (k >> 6, 0);
                }
                mask |= 1 << (k & 63);
                swapped += 1;
            });
        });
        swap(cur, mask);

        swapped
    }
}

/// Multi-point crossover at bit granularity for [`PackedBitChromosome`].
///
/// Same semantics as [`MultiPointCrossover`](crate::MultiPointCrossover): `num_points` cut
/// points are drawn from `1..num_bits`, and the segments between them are alternately
/// swapped, starting with the first. The generic `MultiPointCrossover` can only cut
/// at word boundaries on a packed chromosome. This one can cut at any bit.
///
/// Whole words inside a segment are swapped directly, and the words a cut falls
/// inside are swapped under a mask. The rate decides which pairs are crossed.
#[derive(Debug, Clone)]
pub struct PackedBitMultiPointCrossover {
    rate: Expr,
    num_points: usize,
}

impl PackedBitMultiPointCrossover {
    pub fn new(rate: impl Into<Expr>, num_points: usize) -> Self {
        Self {
            rate: rate.into(),
            num_points,
        }
    }
}

impl Crossover<PackedBitChromosome> for PackedBitMultiPointCrossover {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        one: &mut PackedBitChromosome,
        two: &mut PackedBitChromosome,
        _: &mut AlterContext,
    ) -> usize {
        let num_bits = one.num_bits().min(two.num_bits());
        crossover_multi_point_bits(
            one.as_mut_slice(),
            two.as_mut_slice(),
            num_bits,
            self.num_points,
        )
    }
}

/// Swaps bits `[0, cut_1)`, `[cut_2, cut_3)`, between `a` and `b`, with the
/// cuts drawn from `1..num_bits`. Returns the number of cut points.
#[inline]
pub fn crossover_multi_point_bits(
    a: &mut [BitWordGene],
    b: &mut [BitWordGene],
    num_bits: usize,
    num_points: usize,
) -> usize {
    if num_bits < 2 {
        return 0;
    }

    let num_points = num_points.clamp(1, num_bits - 1);
    let mut cuts = random_provider::sample_indices(1..num_bits, num_points);
    cuts.sort_unstable();

    let mut last = 0;
    for (i, &cut) in cuts.iter().enumerate() {
        if i % 2 == 0 {
            swap_bit_range(a, b, last, cut);
        }
        last = cut;
    }

    if num_points.is_multiple_of(2) {
        swap_bit_range(a, b, last, num_bits);
    }

    num_points
}

/// Swaps bits `[lo, hi)` between `a` and `b`.
#[inline]
fn swap_bit_range(a: &mut [BitWordGene], b: &mut [BitWordGene], lo: usize, hi: usize) {
    if lo >= hi {
        return;
    }

    let (lo_word, hi_word) = (lo >> 6, (hi - 1) >> 6);
    let lo_mask = u64::MAX << (lo & 63);
    let hi_mask = u64::MAX >> (63 - ((hi - 1) & 63));

    if lo_word == hi_word {
        swap_masked(a, b, lo_word, lo_mask & hi_mask);
        return;
    }

    swap_masked(a, b, lo_word, lo_mask);
    a[lo_word + 1..hi_word].swap_with_slice(&mut b[lo_word + 1..hi_word]);
    swap_masked(a, b, hi_word, hi_mask);
}

/// Swaps the bits of word `w` selected by `mask` between `a` and `b`.
#[inline]
fn swap_masked(a: &mut [BitWordGene], b: &mut [BitWordGene], w: usize, mask: u64) {
    let d = (a[w].allele() ^ b[w].allele()) & mask;
    *a[w].allele_mut() ^= d;
    *b[w].allele_mut() ^= d;
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::Chromosome;

    fn packed(bit: bool, num_bits: usize) -> PackedBitChromosome {
        std::iter::repeat_n(bit, num_bits).collect()
    }

    /// Positions `i` where the child switches parent, i.e. `child[i] != child[i - 1]`.
    fn cut_points(child: &PackedBitChromosome) -> Vec<usize> {
        (1..child.num_bits())
            .filter(|&i| child.bit(i) != child.bit(i - 1))
            .collect()
    }

    fn cross(one: &mut PackedBitChromosome, two: &mut PackedBitChromosome, points: usize) -> usize {
        let num_bits = one.num_bits();
        crossover_multi_point_bits(one.as_mut_slice(), two.as_mut_slice(), num_bits, points)
    }

    #[test]
    fn test_multi_point_bits_makes_num_points_cuts() {
        // With parents of all 0s and all 1s, every cut shows up as exactly one switch in
        // the child, and the two children stay complements of each other.
        for num_bits in [2usize, 3, 63, 64, 65, 130, 200] {
            for num_points in [1, 2, 3, 5] {
                let expected = num_points.min(num_bits - 1);
                for _ in 0..200 {
                    let mut one = packed(false, num_bits);
                    let mut two = packed(true, num_bits);

                    let points = cross(&mut one, &mut two, num_points);
                    let cuts = cut_points(&one);

                    assert_eq!(points, expected);
                    assert_eq!(cuts.len(), expected, "num_bits {num_bits}, cuts {cuts:?}");
                    assert_eq!(cut_points(&two), cuts);
                    assert_eq!(one.count_ones() + two.count_ones(), num_bits);
                    assert!((0..num_bits).all(|i| one.bit(i) != two.bit(i)));
                    // The first segment is always swapped.
                    assert!(one.bit(0));
                }
            }
        }
    }

    #[test]
    fn test_multi_point_bits_only_exchanges_bits() {
        for num_bits in [65usize, 128, 300] {
            let (a, b) = (
                PackedBitChromosome::new(num_bits),
                PackedBitChromosome::new(num_bits),
            );
            let (mut one, mut two) = (a.clone(), b.clone());

            cross(&mut one, &mut two, 3);

            for i in 0..num_bits {
                let before = (a.bit(i), b.bit(i));
                let after = (one.bit(i), two.bit(i));
                assert!(after == before || after == (before.1, before.0), "bit {i}");
            }
        }
    }

    #[test]
    fn test_multi_point_bits_cuts_reach_every_position() {
        // 130 bits spans a full word, a word boundary and a partial tail word.
        const NUM_BITS: usize = 130;
        let mut hit = [false; NUM_BITS];
        for _ in 0..20_000 {
            let mut one = packed(false, NUM_BITS);
            let mut two = packed(true, NUM_BITS);
            cross(&mut one, &mut two, 2);
            for cut in cut_points(&one) {
                hit[cut] = true;
            }
        }

        assert!(!hit[0]);
        assert!(
            hit[1..].iter().all(|&h| h),
            "missed cuts: {:?}",
            (1..NUM_BITS).filter(|&i| !hit[i]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_multi_point_bits_short_chromosomes_are_untouched() {
        for num_bits in [0usize, 1] {
            let (mut one, mut two) = (packed(false, num_bits), packed(true, num_bits));
            assert_eq!(cross(&mut one, &mut two, 2), 0);
            assert_eq!(one, packed(false, num_bits));
            assert_eq!(two.len(), num_bits.div_ceil(64));
        }
    }
}
