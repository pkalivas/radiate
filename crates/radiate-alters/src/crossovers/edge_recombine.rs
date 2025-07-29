use radiate_core::{AlterResult, Chromosome, Crossover, PermutationChromosome, random_provider};
use std::collections::HashMap;

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
        used: &[usize],
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
        rate: f32,
    ) -> AlterResult {
        if random_provider::random::<f32>() >= rate {
            return 0.into();
        }

        // Convert chromosomes to permutation representation
        let parent1: Vec<usize> = chrom_one.iter().map(|g| g.index()).collect();
        let parent2: Vec<usize> = chrom_two.iter().map(|g| g.index()).collect();

        let edge_table = self.build_edge_table(&parent1, &parent2);

        // Build offspring
        let mut offspring = Vec::new();
        let mut used = Vec::new();

        // Start with a random element
        let start = parent1[random_provider::range(0..parent1.len())];
        offspring.push(start);
        used.push(start);

        while offspring.len() < parent1.len() {
            if let Some(next) = self.select_next(&edge_table, &used) {
                offspring.push(next);
                used.push(next);
            } else {
                // If no valid next element, pick any unused element
                for i in 0..parent1.len() {
                    if !used.contains(&i) {
                        offspring.push(i);
                        used.push(i);
                        break;
                    }
                }
            }
        }

        // Update chromosomes
        for (i, allele) in offspring.iter().enumerate() {
            chrom_one.set(i, chrom_one.get(i).with_index(*allele));
        }

        1.into()
    }
}
