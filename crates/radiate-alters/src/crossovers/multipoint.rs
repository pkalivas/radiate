use radiate_core::{
    AlterContext, Crossover, Expr, RateSet, chromosomes::ContiguousChromosome, random_provider,
};

/// The [MultiPointCrossover] is a crossover method that takes two chromosomes and crosses them
/// by selecting multiple points in the chromosome and swapping the genes between the two chromosomes.
/// The number of points to swap is determined by the `num_points` parameter and must be between 1 and the
/// length of the chromosome. Note, in most cases having more than 2 points is not useful and actually
/// reduces the effectiveness of the crossover. However, it can be useful in some cases so it is allowed.
///
/// This is the traditional crossver method used by genetic algorithms. It is a
/// simple method that can be used with any type of gene.
pub struct MultiPointCrossover {
    num_points: usize,
    rate: Expr,
}

impl MultiPointCrossover {
    /// Create a new instance of the [MultiPointCrossover] with the given rate and number of points.
    /// The rate must be between 0.0 and 1.0, and the number of points must be between 1 and the length
    /// of the chromosome.
    pub fn new(rate: impl Into<Expr>, num_points: usize) -> Self {
        Self {
            num_points,
            rate: rate.into(),
        }
    }
}

impl<C: ContiguousChromosome> Crossover<C> for MultiPointCrossover {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        _: &mut AlterContext,
    ) -> usize {
        let one = chrom_one.as_mut_slice();
        let two = chrom_two.as_mut_slice();

        if self.num_points == 1 {
            crossover_single_point(one, two)
        } else {
            crossover_multi_point(one, two, self.num_points)
        }
    }
}

#[inline]
pub fn crossover_multi_point<G>(
    chrom_one: &mut [G],
    chrom_two: &mut [G],
    num_points: usize,
) -> usize {
    let length = std::cmp::min(chrom_one.len(), chrom_two.len());

    if length < 2 {
        return 0;
    }

    let num_points = num_points.clamp(1, length - 1);

    // Valid cut points are 1..length - cutting before the first gene would swap an empty segment.
    let mut selected_points = random_provider::sample_indices(1..length, num_points);

    selected_points.sort();

    let mut current_parent = 1;
    let mut last_point = 0;

    for i in selected_points {
        if current_parent == 1 {
            chrom_one[last_point..i].swap_with_slice(&mut chrom_two[last_point..i]);
        }

        // Oscillate between parents
        current_parent = 3 - current_parent;
        last_point = i;
    }

    if current_parent == 1 {
        chrom_one[last_point..].swap_with_slice(&mut chrom_two[last_point..]);
    }

    num_points
}

#[inline]
pub fn crossover_single_point<G>(chrom_one: &mut [G], chrom_two: &mut [G]) -> usize {
    let length = std::cmp::min(chrom_one.len(), chrom_two.len());

    if length < 2 {
        return 0;
    }

    let crossover_point = random_provider::range(1..length);
    chrom_one[crossover_point..].swap_with_slice(&mut chrom_two[crossover_point..]);

    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossover_multi_point() {
        let mut chrom_one = vec![0; 10];
        let mut chrom_two = vec![1; 10];

        let points = crossover_multi_point(&mut chrom_one, &mut chrom_two, 2);

        assert_eq!(chrom_one.len(), 10);
        assert_eq!(chrom_two.len(), 10);
        assert_eq!(points, 2);
    }

    /// Positions `i` where the child switches parent, i.e. `child[i] != child[i - 1]`.
    fn cut_points(child: &[i32]) -> Vec<usize> {
        (1..child.len())
            .filter(|&i| child[i] != child[i - 1])
            .collect()
    }

    #[test]
    fn test_crossover_multi_point_always_makes_num_points_cuts() {
        // With parents of all 0s and all 1s, every cut point shows up as exactly one
        // switch in the child. A cut at index 0 would swap an empty segment and lose a cut.
        for length in [2usize, 3, 10, 50] {
            for num_points in 1..length {
                for _ in 0..200 {
                    let mut one = vec![0; length];
                    let mut two = vec![1; length];

                    let points = crossover_multi_point(&mut one, &mut two, num_points);
                    let cuts = cut_points(&one);

                    assert_eq!(points, num_points);
                    assert_eq!(cuts.len(), num_points, "length {length}, cuts {cuts:?}");
                    assert_eq!(cut_points(&two), cuts);
                }
            }
        }
    }

    #[test]
    fn test_crossover_multi_point_cut_points_are_uniform() {
        const LENGTH: usize = 10;
        const TRIALS: usize = 90_000;

        let mut counts = [0usize; LENGTH];
        for _ in 0..TRIALS {
            let mut one = vec![0; LENGTH];
            let mut two = vec![1; LENGTH];
            crossover_multi_point(&mut one, &mut two, 2);

            for cut in cut_points(&one) {
                counts[cut] += 1;
            }
        }

        // 2 cuts spread over the 9 valid positions 1..10: each is picked with probability 2/9.
        assert_eq!(counts[0], 0);
        let expected = TRIALS as f64 * 2.0 / 9.0;
        for (i, &count) in counts.iter().enumerate().skip(1) {
            let deviation = (count as f64 - expected).abs() / expected;
            assert!(
                deviation < 0.03,
                "cut point {i}: {count} vs ~{expected:.0}, {counts:?}"
            );
        }
    }
}
