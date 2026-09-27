use radiate_core::{
    AlterContext, Chromosome, Crossover, Expr, PermutationChromosome, PermutationGene, RateSet,
    math::indexes,
};

/// Partially Matched Crossover (PMX) for permutations.
///
/// A random segment is exchanged between the two parents. Outside the segment each
/// child keeps its own genes, except where a gene would be duplicated - those are
/// resolved through the mapping defined by the segment, so both children stay valid
/// permutations.
///
/// Implemented in place: each child is built by swapping genes within its own
/// chromosome, using a position lookup per parent, so a crossover is `O(n)` and never
/// clones genes.
pub struct PMXCrossover {
    rate: Expr,
}

impl PMXCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        PMXCrossover { rate: rate.into() }
    }
}

impl<A: PartialEq + Clone> Crossover<PermutationChromosome<A>> for PMXCrossover {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut PermutationChromosome<A>,
        chrom_two: &mut PermutationChromosome<A>,
        _: &mut AlterContext,
    ) -> usize {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return 0;
        }

        debug_assert_eq!(
            chrom_one.len(),
            chrom_two.len(),
            "PMX parents must be permutations of the same alleles"
        );

        // `fill_subset` returns distinct indices in ascending order, so start < end.
        let mut subset = [0; 2];
        indexes::fill_subset(length, &mut subset);
        let (start, end) = (subset[0], subset[1]);

        let num_alleles = chrom_one.alleles().len();
        partially_matched_swap(
            &mut chrom_one.genes,
            &mut chrom_two.genes,
            start,
            end,
            num_alleles,
        );

        2
    }
}

/// Goldberg's swap formulation of PMX: for each position `i` in `start..=end`, child one
/// swaps the gene holding parent two's value at `i` into place, and vice versa.
///
/// The value each child needs must come from a snapshot of the *other parent's original
/// segment*. Reading the other chromosome's current value instead gives a different
/// (non-PMX) result, because earlier swaps can already have moved genes inside the segment.
fn partially_matched_swap<A: PartialEq + Clone>(
    one: &mut [PermutationGene<A>],
    two: &mut [PermutationGene<A>],
    start: usize,
    end: usize,
    num_alleles: usize,
) {
    let segment_len = end - start + 1;

    // One allocation: [position in one | position in two | segment of one | segment of two]
    let mut buffer = vec![0; 2 * num_alleles + 2 * segment_len];
    let (pos_one, rest) = buffer.split_at_mut(num_alleles);
    let (pos_two, rest) = rest.split_at_mut(num_alleles);
    let (segment_one, segment_two) = rest.split_at_mut(segment_len);

    for (i, gene) in one.iter().enumerate() {
        pos_one[gene.index()] = i;
    }

    for (i, gene) in two.iter().enumerate() {
        pos_two[gene.index()] = i;
    }

    for (k, i) in (start..=end).enumerate() {
        segment_one[k] = one[i].index();
        segment_two[k] = two[i].index();
    }

    for (k, i) in (start..=end).enumerate() {
        swap_into_place(one, pos_one, i, segment_two[k]);
        swap_into_place(two, pos_two, i, segment_one[k]);
    }
}

/// Moves the gene holding `value` to position `i`, swapping it with the gene currently
/// there, and keeps `positions` in sync.
#[inline]
fn swap_into_place<A: PartialEq + Clone>(
    genes: &mut [PermutationGene<A>],
    positions: &mut [usize],
    i: usize,
    value: usize,
) {
    let j = positions[value];
    let displaced = genes[i].index();

    genes.swap(i, j);
    positions[displaced] = j;
    positions[value] = i;
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::{Gene, PermutationGene, Valid, alter::AlterUpdates, random_provider};
    use std::sync::Arc;

    const TRIALS: usize = 2_000;

    /// A chromosome over the alleles `0..order.len()`, laid out in `order`.
    fn perm(order: &[usize]) -> PermutationChromosome<usize> {
        let alleles: Arc<[usize]> = (0..order.len()).collect();
        let genes = order
            .iter()
            .map(|&i| PermutationGene::new(i, Arc::clone(&alleles)))
            .collect();
        PermutationChromosome::new(genes, alleles)
    }

    fn random_perm(n: usize) -> Vec<usize> {
        random_provider::shuffled_indices(0..n)
    }

    fn order(chromosome: &PermutationChromosome<usize>) -> Vec<usize> {
        chromosome.genes.iter().map(|g| g.index()).collect()
    }

    fn is_permutation(order: &[usize]) -> bool {
        let mut sorted = order.to_vec();
        sorted.sort_unstable();
        sorted.iter().copied().eq(0..order.len())
    }

    fn cross(one: &[usize], two: &[usize]) -> (Vec<usize>, Vec<usize>, usize) {
        let (mut one, mut two) = (perm(one), perm(two));
        let mut updates = AlterUpdates::default();
        let mut ctx = AlterContext::new(&mut updates, 0, 1.0, &[]);
        let count = PMXCrossover::new(1.0).cross_chromosomes(&mut one, &mut two, &mut ctx);
        (order(&one), order(&two), count)
    }

    /// Textbook PMX: `child` takes `donor[start..=end]`, and every other position keeps
    /// `parent[i]`, following the segment's `donor -> parent` mapping until it no longer
    /// collides with the copied segment.
    fn reference_child(parent: &[usize], donor: &[usize], start: usize, end: usize) -> Vec<usize> {
        let segment = &donor[start..=end];
        (0..parent.len())
            .map(|i| {
                if (start..=end).contains(&i) {
                    return donor[i];
                }
                let mut value = parent[i];
                while let Some(k) = segment.iter().position(|&v| v == value) {
                    value = parent[start + k];
                }
                value
            })
            .collect()
    }

    /// Every window `(start, end)` for which the operator's output is exactly PMX.
    fn matching_windows(
        one: &[usize],
        two: &[usize],
        child_one: &[usize],
        child_two: &[usize],
    ) -> Vec<(usize, usize)> {
        let n = one.len();
        (0..n)
            .flat_map(|start| (start..n).map(move |end| (start, end)))
            .filter(|&(start, end)| {
                reference_child(one, two, start, end) == child_one
                    && reference_child(two, one, start, end) == child_two
            })
            .collect()
    }

    #[test]
    fn reference_matches_textbook_example() {
        // Goldberg & Lingle (1985), 1-based alleles shifted to 0-based, cut around 4..=7.
        let one = [1, 2, 3, 4, 5, 6, 7, 8, 9].map(|v| v - 1);
        let two = [4, 5, 2, 1, 8, 7, 6, 9, 3].map(|v| v - 1);

        assert_eq!(
            reference_child(&one, &two, 3, 6),
            [4, 2, 3, 1, 8, 7, 6, 5, 9].map(|v| v - 1)
        );
        assert_eq!(
            reference_child(&two, &one, 3, 6),
            [1, 8, 2, 4, 5, 6, 7, 9, 3].map(|v| v - 1)
        );
    }

    #[test]
    fn children_are_valid_permutations() {
        random_provider::seed(1);

        for n in [2, 3, 5, 10, 50] {
            for _ in 0..TRIALS / 4 {
                let (one, two, count) = cross(&random_perm(n), &random_perm(n));

                assert_eq!(count, 2);
                assert!(is_permutation(&one), "n = {n}: {one:?}");
                assert!(is_permutation(&two), "n = {n}: {two:?}");
                assert!(perm(&one).is_valid() && perm(&two).is_valid());
            }
        }
    }

    #[test]
    fn children_are_pmx_of_parents_for_some_segment() {
        random_provider::seed(2);

        for n in 2..=12 {
            for _ in 0..TRIALS / 10 {
                let (one, two) = (random_perm(n), random_perm(n));
                let (child_one, child_two, _) = cross(&one, &two);

                assert!(
                    !matching_windows(&one, &two, &child_one, &child_two).is_empty(),
                    "not PMX for any segment:\n  parents  {one:?} x {two:?}\n  children {child_one:?}, {child_two:?}"
                );
            }
        }
    }

    #[test]
    fn every_segment_is_reachable() {
        random_provider::seed(3);

        // Count each window that is the *only* one explaining a crossover. A window
        // covering `n - 1` or `n` positions forces the remaining gene too, so those all
        // produce a plain swap of the parents and are counted together as one outcome.
        // Every shorter `start < end` window should appear, including ones touching either end.
        let n = 6;
        let mut seen = vec![vec![0usize; n]; n];
        let mut full_swaps = 0;
        for _ in 0..TRIALS * 5 {
            let (one, two) = (random_perm(n), random_perm(n));
            let (child_one, child_two, _) = cross(&one, &two);

            if let [(start, end)] = matching_windows(&one, &two, &child_one, &child_two)[..] {
                seen[start][end] += 1;
            } else if child_one == two && child_two == one {
                full_swaps += 1;
            }
        }

        for start in 0..n {
            for end in (start + 1..n).filter(|end| end - start < n - 2) {
                assert!(
                    seen[start][end] > 0,
                    "segment {start}..={end} never chosen: {seen:?}"
                );
            }
        }
        assert!(
            full_swaps > 0,
            "no segment of n - 1 or more positions chosen"
        );
    }

    #[test]
    fn positions_where_parents_agree_are_kept() {
        random_provider::seed(4);

        for _ in 0..TRIALS {
            let one = random_perm(10);
            // Swap two positions so the parents agree everywhere else.
            let mut two = one.clone();
            let (a, b) = (random_provider::range(0..10), random_provider::range(0..10));
            two.swap(a, b);

            let (child_one, child_two, _) = cross(&one, &two);
            for i in (0..10).filter(|&i| one[i] == two[i]) {
                assert_eq!(child_one[i], one[i], "position {i}");
                assert_eq!(child_two[i], two[i], "position {i}");
            }
        }
    }

    #[test]
    fn identical_parents_are_unchanged() {
        random_provider::seed(5);

        for n in [2, 7, 30] {
            let parent = random_perm(n);
            let (one, two, _) = cross(&parent, &parent);
            assert_eq!(one, parent);
            assert_eq!(two, parent);
        }
    }

    #[test]
    fn short_chromosomes_are_left_alone() {
        for order in [&[][..], &[0][..]] {
            let (one, two, count) = cross(order, order);
            assert_eq!(count, 0);
            assert_eq!(one, order);
            assert_eq!(two, order);
        }
    }

    #[test]
    fn works_with_non_numeric_alleles() {
        random_provider::seed(6);

        let alleles: Arc<[&str]> = ["a", "b", "c", "d", "e", "f", "g"].into();
        let build = |order: &[usize]| {
            let genes = order
                .iter()
                .map(|&i| PermutationGene::new(i, Arc::clone(&alleles)))
                .collect();
            PermutationChromosome::new(genes, Arc::clone(&alleles))
        };

        for _ in 0..TRIALS / 10 {
            let (mut one, mut two) = (build(&random_perm(7)), build(&random_perm(7)));
            let mut updates = AlterUpdates::default();
            let mut ctx = AlterContext::new(&mut updates, 0, 1.0, &[]);
            PMXCrossover::new(1.0).cross_chromosomes(&mut one, &mut two, &mut ctx);

            for child in [&one, &two] {
                let mut values: Vec<&str> = child.genes.iter().map(|g| *g.allele()).collect();
                values.sort_unstable();
                assert_eq!(values, ["a", "b", "c", "d", "e", "f", "g"]);
            }
        }
    }
}
