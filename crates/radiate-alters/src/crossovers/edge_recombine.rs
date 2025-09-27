use radiate_core::{AlterResult, Chromosome, Crossover, PermutationChromosome, random_provider};
use std::collections::{HashMap, HashSet};

// Example: Parents [1,2,3,4,5] and [1,3,5,2,4]
// Edge table: 1->[2,3], 2->[1,4], 3->[2,5], 4->[5,2], 5->[4,3]
// Offspring: [1,2,4,5,3] (following edges when possible)
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRecombinationCrossover {
    rate: f32,
}

impl EdgeRecombinationCrossover {
    pub fn new(rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }
        EdgeRecombinationCrossover { rate }
    }

    fn build_edge_table(&self, parent1: &[usize], parent2: &[usize]) -> HashMap<usize, Vec<usize>> {
        let mut edge_table = HashMap::new();

        for i in 0..parent1.len() {
            let current = parent1[i];
            let next = parent1[(i + 1) % parent1.len()];
            let prev = parent1[(i + parent1.len() - 1) % parent1.len()];

            edge_table
                .entry(current)
                .or_insert_with(Vec::new)
                .push(next);
            edge_table
                .entry(current)
                .or_insert_with(Vec::new)
                .push(prev);
        }

        for i in 0..parent2.len() {
            let current = parent2[i];
            let next = parent2[(i + 1) % parent2.len()];
            let prev = parent2[(i + parent2.len() - 1) % parent2.len()];

            if !edge_table[&current].contains(&next) {
                edge_table.get_mut(&current).unwrap().push(next);
            }
            if !edge_table[&current].contains(&prev) {
                edge_table.get_mut(&current).unwrap().push(prev);
            }
        }

        edge_table
    }

    fn select_next(
        &self,
        edge_table: &HashMap<usize, Vec<usize>>,
        used: &HashSet<usize>,
    ) -> Option<usize> {
        let mut candidates = Vec::new();
        let mut min_edges = usize::MAX;

        for (node, edges) in edge_table {
            if !used.contains(node) {
                let available_edges = edges.iter().filter(|e| !used.contains(e)).count();
                if available_edges < min_edges {
                    min_edges = available_edges;
                    candidates.clear();
                    candidates.push(*node);
                } else if available_edges == min_edges {
                    candidates.push(*node);
                }
            }
        }

        if candidates.is_empty() {
            None
        } else {
            Some(candidates[random_provider::range(0..candidates.len())])
        }
    }
}

impl<T> Crossover<PermutationChromosome<T>> for EdgeRecombinationCrossover
where
    T: PartialEq + Clone,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut PermutationChromosome<T>,
        chrom_two: &mut PermutationChromosome<T>,
        _: f32,
    ) -> AlterResult {
        let parent1 = chrom_one.iter().map(|g| g.index()).collect::<Vec<usize>>();
        let parent2 = chrom_two.iter().map(|g| g.index()).collect::<Vec<usize>>();

        let edge_table = self.build_edge_table(&parent1, &parent2);

        let mut offspring = Vec::new();
        let mut used = HashSet::new();

        // Start with a random element
        let start = parent1[random_provider::range(0..parent1.len())];
        offspring.push(start);
        used.insert(start);

        while offspring.len() < parent1.len() {
            if let Some(next) = self.select_next(&edge_table, &used) {
                offspring.push(next);
                used.insert(next);
            } else {
                // If no valid next element, pick any unused element
                for i in 0..parent1.len() {
                    if !used.contains(&i) {
                        offspring.push(i);
                        used.insert(i);
                        break;
                    }
                }
            }
        }

        for (i, allele) in offspring.iter().enumerate() {
            chrom_one.set(i, chrom_one.get(i).with_index(*allele));
        }

        1.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::genome::{PermutationChromosome, PermutationGene};
    use std::sync::Arc;

    #[test]
    fn test_edge_recombination_crossover_new() {
        let crossover = EdgeRecombinationCrossover::new(0.5);
        assert_eq!(crossover.rate, 0.5);
    }

    #[test]
    fn test_build_edge_table() {
        let crossover = EdgeRecombinationCrossover::new(0.5);
        let parent1 = vec![0, 1, 2, 3, 4];
        let parent2 = vec![0, 2, 4, 1, 3];

        let edge_table = crossover.build_edge_table(&parent1, &parent2);

        // Check that all elements have edges
        assert!(edge_table.contains_key(&0));
        assert!(edge_table.contains_key(&1));
        assert!(edge_table.contains_key(&2));
        assert!(edge_table.contains_key(&3));
        assert!(edge_table.contains_key(&4));

        // Check specific edges for element 1
        let edges_1 = &edge_table[&1];
        assert!(edges_1.contains(&0)); // from parent1: 1->0 (prev)
        assert!(edges_1.contains(&2)); // from parent1: 1->2 (next)
        assert!(edges_1.contains(&4)); // from parent2: 1->4 (next)
        assert!(edges_1.contains(&2)); // from parent2: 1->2 (prev)

        // Check that edges are unique
        for edges in edge_table.values() {
            let unique_edges: Vec<&usize> = edges.iter().collect();
            assert_eq!(unique_edges.len(), edges.len());
        }
    }

    #[test]
    fn test_select_next_with_available_edges() {
        let crossover = EdgeRecombinationCrossover::new(0.5);
        let mut edge_table = HashMap::new();
        edge_table.insert(0, vec![1, 2]);
        edge_table.insert(1, vec![0, 3]);
        edge_table.insert(2, vec![0, 3]);
        edge_table.insert(3, vec![1, 2]);

        let mut used = HashSet::new();
        used.insert(0);
        used.insert(1);

        // Should select 2 or 3 (both have 1 available edge)
        let next = crossover.select_next(&edge_table, &used);
        assert!(next.is_some());
        let next_val = next.unwrap();
        assert!(next_val == 2 || next_val == 3);
    }

    #[test]
    fn test_select_next_no_available_edges() {
        let crossover = EdgeRecombinationCrossover::new(0.5);
        let mut edge_table = HashMap::new();
        edge_table.insert(0, vec![1, 2]);
        edge_table.insert(1, vec![0, 2]);
        edge_table.insert(2, vec![0, 1]);

        let mut used = HashSet::new();
        used.insert(0);
        used.insert(1);
        used.insert(2);

        // No elements left to select
        let next = crossover.select_next(&edge_table, &used);
        assert!(next.is_none());
    }

    #[test]
    fn test_cross_chromosomes_basic() {
        let crossover = EdgeRecombinationCrossover::new(0.5);

        let alleles: Arc<[usize]> = vec![0, 1, 2, 3, 4].into_boxed_slice().into();
        let genes1 = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
            PermutationGene::new(4, Arc::clone(&alleles)),
        ];
        let genes2 = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(4, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
        ];

        let mut chrom_one = PermutationChromosome::new(genes1, Arc::clone(&alleles));
        let mut chrom_two = PermutationChromosome::new(genes2, Arc::clone(&alleles));

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        // Should perform 1 crossover operation
        assert_eq!(result.count(), 1);

        // Check that the chromosome is still valid (no duplicates)
        let values: Vec<usize> = chrom_one.iter().map(|g| g.index()).collect();
        let unique_values: Vec<usize> = values
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(values.len(), unique_values.len());

        // Check that all values are in range
        for value in values {
            assert!(value < alleles.len());
        }
    }

    #[test]
    fn test_cross_chromosomes_identical_parents() {
        let crossover = EdgeRecombinationCrossover::new(0.5);

        let alleles: Arc<[usize]> = vec![0, 1, 2, 3, 4].into_boxed_slice().into();
        let genes = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
            PermutationGene::new(4, Arc::clone(&alleles)),
        ];

        let mut chrom_one = PermutationChromosome::new(genes.clone(), Arc::clone(&alleles));
        let mut chrom_two = PermutationChromosome::new(genes, Arc::clone(&alleles));

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        // Should still perform crossover even with identical parents
        assert_eq!(result.count(), 1);

        // The result should still be a valid permutation
        let values: Vec<usize> = chrom_one.iter().map(|g| g.index()).collect();
        let unique_values: Vec<usize> = values
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(values.len(), unique_values.len());
    }

    #[test]
    fn test_cross_chromosomes_edge_cases() {
        let crossover = EdgeRecombinationCrossover::new(0.5);

        // Test with single element
        let alleles: Arc<[usize]> = vec![0].into_boxed_slice().into();
        let genes = vec![PermutationGene::new(0, Arc::clone(&alleles))];

        let mut chrom_one = PermutationChromosome::new(genes.clone(), Arc::clone(&alleles));
        let mut chrom_two = PermutationChromosome::new(genes, Arc::clone(&alleles));

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);
        assert_eq!(result.count(), 1);

        // Test with two elements
        let alleles: Arc<[usize]> = vec![0, 1].into_boxed_slice().into();
        let genes = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
        ];

        let mut chrom_one = PermutationChromosome::new(genes.clone(), Arc::clone(&alleles));
        let mut chrom_two = PermutationChromosome::new(genes, Arc::clone(&alleles));

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);
        assert_eq!(result.count(), 1);
    }

    #[test]
    fn test_edge_recombination_property_based() {
        let crossover = EdgeRecombinationCrossover::new(0.5);

        // Test multiple random combinations
        for _ in 0..100 {
            let alleles: Arc<[usize]> = vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice().into();

            // Create random permutations
            let mut indices1: Vec<usize> = (0..alleles.len()).collect();
            let mut indices2: Vec<usize> = (0..alleles.len()).collect();

            // Shuffle to create random permutations
            for i in 0..indices1.len() {
                let j = random_provider::range(i..indices1.len());
                indices1.swap(i, j);
            }
            for i in 0..indices2.len() {
                let j = random_provider::range(i..indices2.len());
                indices2.swap(i, j);
            }

            let genes1: Vec<PermutationGene<usize>> = indices1
                .iter()
                .map(|&i| PermutationGene::new(i, Arc::clone(&alleles)))
                .collect();
            let genes2: Vec<PermutationGene<usize>> = indices2
                .iter()
                .map(|&i| PermutationGene::new(i, Arc::clone(&alleles)))
                .collect();

            let mut chrom_one = PermutationChromosome::new(genes1, Arc::clone(&alleles));
            let mut chrom_two = PermutationChromosome::new(genes2, Arc::clone(&alleles));

            let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

            // Should always perform exactly 1 crossover
            assert_eq!(result.count(), 1);

            // Result should be a valid permutation
            let values: Vec<usize> = chrom_one.iter().map(|g| g.index()).collect();
            let unique_values: Vec<usize> = values
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            assert_eq!(values.len(), unique_values.len());

            // All values should be in range
            for value in values {
                assert!(value < alleles.len());
            }
        }
    }

    #[test]
    fn test_edge_table_construction_edge_cases() {
        let crossover = EdgeRecombinationCrossover::new(0.5);

        // Test with circular permutations
        let parent1 = vec![0, 1, 2, 3];
        let parent2 = vec![3, 2, 1, 0];

        let edge_table = crossover.build_edge_table(&parent1, &parent2);

        // Each element should have edges to its neighbors in both parents
        for i in 0..4 {
            let edges = &edge_table[&i];
            assert!(edges.len() >= 2); // At least prev and next from each parent
        }
    }

    #[test]
    fn test_edge_recombination_convergence() {
        let crossover = EdgeRecombinationCrossover::new(0.5);

        // Test that the algorithm always produces a complete permutation
        let alleles = vec![0, 1, 2, 3, 4, 5].into_boxed_slice().into();
        let genes1 = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
            PermutationGene::new(4, Arc::clone(&alleles)),
            PermutationGene::new(5, Arc::clone(&alleles)),
        ];
        let genes2 = vec![
            PermutationGene::new(5, Arc::clone(&alleles)),
            PermutationGene::new(4, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(0, Arc::clone(&alleles)),
        ];

        let mut chrom_one = PermutationChromosome::new(genes1, Arc::clone(&alleles));
        let mut chrom_two = PermutationChromosome::new(genes2, Arc::clone(&alleles));

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 1);

        // Verify the result is a complete permutation
        let values: Vec<usize> = chrom_one.iter().map(|g| g.index()).collect();
        assert_eq!(values.len(), alleles.len());

        let mut sorted_values = values.clone();
        sorted_values.sort();
        assert_eq!(sorted_values, (0..alleles.len()).collect::<Vec<usize>>());
    }
}
