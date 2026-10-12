use radiate_core::{
    AlterContext, Chromosome, Crossover, Expr, PermutationChromosome, PermutationGene, RateSet,
    math::indexes,
};

/// Marks an allele that hasn't been given a position in the child yet.
const UNPLACED: usize = usize::MAX;

/// Order Crossover (OX1, Davis 1985) for permutations.
///
/// Each child keeps a random segment of its own parent in place, then fills the remaining
/// positions - starting just after the segment and wrapping around with the missing genes
/// in the order they appear in the other parent, also read from just after the segment.
///
/// Where PMX preserves *absolute positions* and edge recombination preserves *adjacency*,
/// OX preserves *relative order*: the operator of choice for scheduling and sequencing
/// problems where "a comes before b" matters more than where either one sits.
///
/// Example: parents `[1,2,3,4,5,6,7,8,9]` and `[4,5,2,1,8,7,6,9,3]`, segment `3..=6`
/// ```text
/// child one: [2,1,8 | 4,5,6,7 | 9,3]
/// child two: [3,4,5 | 1,8,7,6 | 9,2]
/// ```
///
/// Implemented in place: each child is a reordering of its own parent's genes, so both
/// target orders are computed from the original parents up front, then each chromosome
/// is sorted into place. A crossover never clones genes.
pub struct OrderCrossover {
    rate: Expr,
}

impl OrderCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        OrderCrossover { rate: rate.into() }
    }
}

impl<A: PartialEq + Clone> Crossover<PermutationChromosome<A>> for OrderCrossover {
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
            "OX parents must be permutations of the same alleles"
        );

        // `fill_subset` returns distinct indices in ascending order, so start < end.
        let mut subset = [0; 2];
        indexes::fill_subset(length, &mut subset);
        let (start, end) = (subset[0], subset[1]);

        // One allocation: [position in child one | position in child two], by allele index.
        // Both are computed before either chromosome is reordered.
        let num_alleles = chrom_one.alleles().len();
        let mut buffer = vec![UNPLACED; 2 * num_alleles];
        let (rank_one, rank_two) = buffer.split_at_mut(num_alleles);

        order_ranks(&chrom_one.genes, &chrom_two.genes, start, end, rank_one);
        order_ranks(&chrom_two.genes, &chrom_one.genes, start, end, rank_two);

        chrom_one
            .genes
            .sort_unstable_by_key(|gene| rank_one[gene.index()]);
        chrom_two
            .genes
            .sort_unstable_by_key(|gene| rank_two[gene.index()]);

        2
    }
}

/// Fills `ranks[allele]` with that allele's position in the OX child of `parent` and
/// `donor`: `parent[start..=end]` stays in place, and the rest of the positions, from
/// `end + 1` wrapping around, take the donor's remaining genes in the donor's order from
/// `end + 1`. `ranks` must arrive filled with [`UNPLACED`].
fn order_ranks<A: PartialEq + Clone>(
    parent: &[PermutationGene<A>],
    donor: &[PermutationGene<A>],
    start: usize,
    end: usize,
    ranks: &mut [usize],
) {
    let n = parent.len();

    for i in start..=end {
        ranks[parent[i].index()] = i;
    }

    let mut slot = (end + 1) % n;
    for offset in 1..=n {
        let value = donor[(end + offset) % n].index();
        if ranks[value] == UNPLACED {
            ranks[value] = slot;
            slot = (slot + 1) % n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::{Gene, Valid, alter::AlterUpdates, random_provider};
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
        let count = OrderCrossover::new(1.0).cross_chromosomes(&mut one, &mut two, &mut ctx);
        (order(&one), order(&two), count)
    }

    /// Textbook OX1, written directly from the definition: keep `parent[start..=end]`,
    /// then write the donor's remaining genes, read from `end + 1` with wraparound, into
    /// the free positions from `end + 1` with wraparound.
    fn reference_child(parent: &[usize], donor: &[usize], start: usize, end: usize) -> Vec<usize> {
        let n = parent.len();
        let segment = &parent[start..=end];

        let mut child = vec![usize::MAX; n];
        child[start..=end].copy_from_slice(segment);

        let fill = (1..=n)
            .map(|offset| donor[(end + offset) % n])
            .filter(|v| !segment.contains(v));
        let free = (1..=n)
            .map(|offset| (end + offset) % n)
            .filter(|i| !(start..=end).contains(i));

        for (i, value) in free.zip(fill) {
            child[i] = value;
        }
        child
    }

    /// Every window `(start, end)` for which the operator's output is exactly OX1.
    fn matching_windows(
        one: &[usize],
        two: &[usize],
        child_one: &[usize],
        child_two: &[usize],
    ) -> Vec<(usize, usize)> {
        let n = one.len();
        (0..n)
            .flat_map(|start| (start + 1..n).map(move |end| (start, end)))
            .filter(|&(start, end)| {
                reference_child(one, two, start, end) == child_one
                    && reference_child(two, one, start, end) == child_two
            })
            .collect()
    }

    #[test]
    fn reference_matches_textbook_example() {
        // Davis (1985) as given by Michalewicz, 1-based alleles shifted to 0-based, cut 3..=6.
        let one = [1, 2, 3, 4, 5, 6, 7, 8, 9].map(|v| v - 1);
        let two = [4, 5, 2, 1, 8, 7, 6, 9, 3].map(|v| v - 1);

        assert_eq!(
            reference_child(&one, &two, 3, 6),
            [2, 1, 8, 4, 5, 6, 7, 9, 3].map(|v| v - 1)
        );
        assert_eq!(
            reference_child(&two, &one, 3, 6),
            [3, 4, 5, 1, 8, 7, 6, 9, 2].map(|v| v - 1)
        );
    }

    #[test]
    fn ranks_match_reference() {
        random_provider::seed(1);

        for n in 2..=12 {
            for _ in 0..TRIALS / 10 {
                let (one, two) = (random_perm(n), random_perm(n));
                let (p_one, p_two) = (perm(&one), perm(&two));
                let start = random_provider::range(0..n - 1);
                let end = random_provider::range(start + 1..n);

                let mut ranks = vec![UNPLACED; n];
                order_ranks(&p_one.genes, &p_two.genes, start, end, &mut ranks);

                let mut child = vec![0; n];
                for (allele, &rank) in ranks.iter().enumerate() {
                    child[rank] = allele;
                }
                assert_eq!(child, reference_child(&one, &two, start, end));
            }
        }
    }

    #[test]
    fn children_are_valid_permutations() {
        random_provider::seed(2);

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
    fn children_are_ox_of_parents_for_some_segment() {
        random_provider::seed(3);

        for n in 2..=12 {
            for _ in 0..TRIALS / 10 {
                let (one, two) = (random_perm(n), random_perm(n));
                let (child_one, child_two, _) = cross(&one, &two);

                assert!(
                    !matching_windows(&one, &two, &child_one, &child_two).is_empty(),
                    "not OX for any segment:\n  parents  {one:?} x {two:?}\n  children {child_one:?}, {child_two:?}"
                );
            }
        }
    }

    #[test]
    fn every_segment_is_reachable() {
        random_provider::seed(4);

        // Count each window that is the *only* one explaining a crossover. A window
        // covering `n - 1` or `n` positions leaves at most one free slot, so the children
        // come back unchanged - those are counted together as one outcome.
        let n = 6;
        let mut seen = vec![vec![0usize; n]; n];
        let mut unchanged = 0;
        for _ in 0..TRIALS * 5 {
            let (one, two) = (random_perm(n), random_perm(n));
            let (child_one, child_two, _) = cross(&one, &two);

            if let [(start, end)] = matching_windows(&one, &two, &child_one, &child_two)[..] {
                seen[start][end] += 1;
            } else if child_one == one && child_two == two {
                unchanged += 1;
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
            unchanged > 0,
            "no segment of n - 1 or more positions chosen"
        );
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
    fn same_seed_same_offspring() {
        let one = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let two = [9, 3, 7, 1, 5, 0, 8, 2, 6, 4];

        let first = random_provider::scoped_seed(11, || cross(&one, &two));
        let second = random_provider::scoped_seed(11, || cross(&one, &two));
        assert_eq!(first, second);
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
            OrderCrossover::new(1.0).cross_chromosomes(&mut one, &mut two, &mut ctx);

            for child in [&one, &two] {
                let mut values: Vec<&str> = child.genes.iter().map(|g| *g.allele()).collect();
                values.sort_unstable();
                assert_eq!(values, ["a", "b", "c", "d", "e", "f", "g"]);
            }
        }
    }
}
