use radiate_core::{
    AlterContext, Chromosome, ContiguousChromosome, Crossover, Expr, PermutationChromosome,
    PermutationGene, RateSet, RdRand, random_provider,
};

/// Each node has at most two neighbours per parent, so at most four in the union.
const MAX_EDGES: usize = 4;

/// Edge Recombination Crossover (ERX, Whitley et al.). Builds an offspring that inherits
/// as many adjacencies as possible from both parents - the operator of choice for
/// tour-like permutations (TSP, routing) where *which genes are neighbours* matters
/// more than where they sit.
///
/// Starting from a random node, the walk repeatedly moves to the current node's unvisited
/// neighbour (in either parent) with the fewest unvisited neighbours of its own, breaking
/// ties at random. Only when the current node has no unvisited neighbours left does it
/// jump to a random unvisited node.
///
/// Example: parents `[1,2,3,4,5]` and `[1,3,5,2,4]`
/// ```text
/// edge table: 1->[2,5,3,4]  2->[1,3,5,4]  3->[2,4,1,5]  4->[3,5,2,1]  5->[4,1,3,2]
/// ```
///
/// The offspring replaces the first chromosome; the second is left untouched.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRecombinationCrossover {
    rate: Expr,
}

impl EdgeRecombinationCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        EdgeRecombinationCrossover { rate: rate.into() }
    }
}

impl<T> Crossover<PermutationChromosome<T>> for EdgeRecombinationCrossover
where
    T: PartialEq + Clone,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut PermutationChromosome<T>,
        chrom_two: &mut PermutationChromosome<T>,
        _: &mut AlterContext,
    ) -> usize {
        let n = chrom_one.len();
        debug_assert_eq!(
            n,
            chrom_two.len(),
            "parents must be permutations of the same set"
        );

        if n < 2 {
            return 0;
        }

        let mut table = build_edge_table([chrom_one.as_slice(), chrom_two.as_slice()]);

        random_provider::with_rng(|rand| {
            let mut current = chrom_one.as_slice()[rand.range(0..n)].index();
            place(&mut table, current, 0);

            for rank in 1..n {
                current = select_next(&table, current, n - rank, rand);
                place(&mut table, current, rank);
            }
        });

        // The offspring is a reordering of chrom_one's own genes, so sort them into place
        // rather than rebuilding each gene.
        chrom_one
            .as_mut_slice()
            .sort_unstable_by_key(|gene| table[gene.index()].rank);

        1
    }
}

/// One row of the edge table, indexed by [`PermutationGene::index`].
#[derive(Clone, Copy, Default)]
struct Node {
    edges: [usize; MAX_EDGES],
    len: u8,
    /// Neighbours not yet placed in the offspring. Edges are symmetric, so placing a node
    /// decrements this on each of its neighbours.
    degree: u8,
    used: bool,
    /// Position in the offspring, once placed.
    rank: usize,
}

impl Node {
    fn edges(&self) -> &[usize] {
        &self.edges[..self.len as usize]
    }

    fn link(&mut self, other: usize) {
        if !self.edges().contains(&other) {
            self.edges[self.len as usize] = other;
            self.len += 1;
        }
    }
}

/// Union of both parents' cyclic adjacency, one [`Node`] per gene index.
fn build_edge_table<T: PartialEq + Clone>(parents: [&[PermutationGene<T>]; 2]) -> Vec<Node> {
    let n = parents[0].len();
    let mut table = vec![Node::default(); n];

    for parent in parents {
        for i in 0..n {
            let current = parent[i].index();
            let next = parent[(i + 1) % n].index();
            table[current].link(next);
            table[next].link(current);
        }
    }

    for node in &mut table {
        node.degree = node.len;
    }

    table
}

/// Mark `node` as placed at `rank` and drop it from its neighbours' degrees.
fn place(table: &mut [Node], node: usize, rank: usize) {
    let Node { edges, len, .. } = table[node];
    table[node].used = true;
    table[node].rank = rank;

    for &neighbour in &edges[..len as usize] {
        table[neighbour].degree -= 1;
    }
}

/// The unused neighbour of `current` with the fewest unused neighbours (ties broken at
/// random), or a random unused node if `current` has none. `remaining` is the number
/// of unused nodes and must be non-zero.
fn select_next(table: &[Node], current: usize, remaining: usize, rand: &mut RdRand) -> usize {
    let mut candidates = [0; MAX_EDGES];
    let mut count = 0;
    let mut min_degree = u8::MAX;

    for &neighbour in table[current].edges() {
        let node = &table[neighbour];
        if node.used {
            continue;
        }

        if node.degree < min_degree {
            min_degree = node.degree;
            count = 0;
        }

        if node.degree == min_degree {
            candidates[count] = neighbour;
            count += 1;
        }
    }

    match count {
        0 => {
            let pick = rand.range(0..remaining);
            table
                .iter()
                .enumerate()
                .filter(|(_, node)| !node.used)
                .nth(pick)
                .map(|(index, _)| index)
                .expect("remaining must match the number of unused nodes")
        }
        1 => candidates[0],
        _ => candidates[rand.range(0..count)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::alter::AlterUpdates;
    use std::sync::Arc;

    fn chromosome(order: &[usize]) -> PermutationChromosome<usize> {
        let alleles: Arc<[usize]> = (0..order.len()).collect::<Vec<_>>().into();
        let genes = order
            .iter()
            .map(|&i| PermutationGene::new(i, Arc::clone(&alleles)))
            .collect();
        PermutationChromosome::new(genes, alleles)
    }

    fn cross(one: &[usize], two: &[usize]) -> (Vec<usize>, usize) {
        let (mut chrom_one, mut chrom_two) = (chromosome(one), chromosome(two));
        let mut updates = AlterUpdates::default();
        let mut ctx = AlterContext::new(&mut updates, 0, 1.0, &[]);

        let count = EdgeRecombinationCrossover::new(1.0).cross_chromosomes(
            &mut chrom_one,
            &mut chrom_two,
            &mut ctx,
        );

        (chrom_one.iter().map(|g| g.index()).collect(), count)
    }

    fn cyclic_edges(order: &[usize]) -> Vec<(usize, usize)> {
        let n = order.len();
        (0..n)
            .map(|i| {
                let (a, b) = (order[i], order[(i + 1) % n]);
                (a.min(b), a.max(b))
            })
            .collect()
    }

    fn assert_permutation(values: &[usize]) {
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..values.len()).collect::<Vec<_>>());
    }

    #[test]
    fn edge_table_is_symmetric_union_of_parent_cycles() {
        let (one, two) = (chromosome(&[0, 1, 2, 3, 4]), chromosome(&[0, 2, 4, 1, 3]));
        let table = build_edge_table([one.as_slice(), two.as_slice()]);

        let mut edges_1 = table[1].edges().to_vec();
        edges_1.sort_unstable();
        assert_eq!(edges_1, vec![0, 2, 3, 4]);

        for (node, row) in table.iter().enumerate() {
            assert_eq!(row.degree, row.len);
            for &neighbour in row.edges() {
                assert!(table[neighbour].edges().contains(&node));
            }
        }
    }

    #[test]
    fn edge_table_deduplicates_shared_edges() {
        let (one, two) = (chromosome(&[0, 1, 2, 3]), chromosome(&[3, 2, 1, 0]));
        let table = build_edge_table([one.as_slice(), two.as_slice()]);

        // Reversed tour shares every edge, so each node keeps exactly two neighbours.
        assert!(table.iter().all(|row| row.len == 2));
    }

    /// 0 - 1, 0 - 2, 2 - 3, 2 - 4: node 2 is a hub, node 1 a leaf.
    fn star_table() -> Vec<Node> {
        let mut table = vec![Node::default(); 5];
        for (a, b) in [(0, 1), (0, 2), (2, 3), (2, 4)] {
            table[a].link(b);
            table[b].link(a);
        }
        for node in &mut table {
            node.degree = node.len;
        }
        table
    }

    #[test]
    fn select_next_prefers_fewest_remaining_edges() {
        let mut table = star_table();
        place(&mut table, 0, 0);

        // From 0: leaf 1 has no unused neighbours left, hub 2 still has two.
        assert_eq!(table[1].degree, 0);
        assert_eq!(table[2].degree, 2);
        let next = random_provider::with_rng(|rand| select_next(&table, 0, 4, rand));
        assert_eq!(next, 1);
    }

    #[test]
    fn select_next_falls_back_to_an_unused_node() {
        let mut table = star_table();
        place(&mut table, 0, 0);
        place(&mut table, 1, 1);

        // 1's only neighbour is used, so the walk must jump.
        for _ in 0..20 {
            let next = random_provider::with_rng(|rand| select_next(&table, 1, 3, rand));
            assert!([2, 3, 4].contains(&next), "jumped to used node {next}");
        }
    }

    #[test]
    fn identical_parents_reproduce_the_tour() {
        let parent = [3, 0, 4, 1, 5, 2, 6];
        for _ in 0..50 {
            let (child, count) = cross(&parent, &parent);
            assert_eq!(count, 1);

            let mut expected = cyclic_edges(&parent);
            let mut actual = cyclic_edges(&child);
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(actual, expected, "child {child:?} broke a parent edge");
        }
    }

    #[test]
    fn offspring_are_valid_permutations() {
        for _ in 0..100 {
            let one = random_provider::shuffled_indices(0..8);
            let two = random_provider::shuffled_indices(0..8);

            let (child, count) = cross(&one, &two);
            assert_eq!(count, 1);
            assert_permutation(&child);
        }
    }

    #[test]
    fn tiny_chromosomes() {
        assert_eq!(cross(&[0], &[0]), (vec![0], 0));

        let (child, count) = cross(&[0, 1], &[1, 0]);
        assert_eq!(count, 1);
        assert_permutation(&child);
    }

    #[test]
    fn same_seed_same_offspring() {
        let one = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let two = [9, 3, 7, 1, 5, 0, 8, 2, 6, 4];

        let first = random_provider::scoped_seed(11, || cross(&one, &two));
        let second = random_provider::scoped_seed(11, || cross(&one, &two));
        assert_eq!(first, second);
    }
}
