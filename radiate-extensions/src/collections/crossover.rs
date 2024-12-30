use std::collections::HashMap;

use crate::collections::{GraphChromosome, NodeType};
use radiate::alter::AlterType;
use radiate::engines::alterers::Alter;
use radiate::engines::genome::*;
use radiate::timer::Timer;
use radiate::{random_provider, Metric};

use super::TreeChromosome;

const NUM_PARENTS: usize = 2;

pub struct GraphCrossover<T>
where
    T: Clone + PartialEq + Default,
{
    pub crossover_rate: f32,
    pub crossover_parent_node_rate: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> GraphCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub fn new(crossover_rate: f32, crossover_parent_node_rate: f32) -> Self {
        Self {
            crossover_rate,
            crossover_parent_node_rate,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn cross(
        &self,
        population: &Population<GraphChromosome<T>>,
        indexes: &[usize],
        generation: i32,
    ) -> Option<Phenotype<GraphChromosome<T>>> {
        let parent_one = &population[indexes[0]];
        let parent_two = &population[indexes[1]];

        let geno_one = parent_one.genotype();
        let geno_two = parent_two.genotype();

        let chromo_index =
            random_provider::random::<usize>() % std::cmp::min(geno_one.len(), geno_two.len());

        let chromo_one = &geno_one[chromo_index];
        let chromo_two = &geno_two[chromo_index];

        let mut new_chromo_one = chromo_one.clone();
        let mut num_crosses = 0;

        let edge_indexes = (0..std::cmp::min(chromo_one.len(), chromo_two.len()))
            .filter(|i| {
                let node_one = chromo_one.get_gene(*i);
                let node_two = chromo_two.get_gene(*i);

                node_one.node_type == NodeType::Edge && node_two.node_type == NodeType::Edge
            })
            .collect::<Vec<usize>>();

        if edge_indexes.is_empty() {
            return None;
        }

        for i in edge_indexes {
            let node_one = chromo_one.get_gene(i);
            let node_two = chromo_two.get_gene(i);

            if random_provider::random::<f32>() < self.crossover_parent_node_rate {
                new_chromo_one.set_gene(node_one.index, node_one.with_allele(node_two.allele()));
                num_crosses += 1;
            }
        }

        if num_crosses > 0 {
            let new_genotype_one = Genotype {
                chromosomes: vec![new_chromo_one],
            };
            let new_phenotype = Phenotype::from_genotype(new_genotype_one, generation);

            return Some(new_phenotype);
        }

        None
    }

    pub fn distinct_subset(limit: usize) -> Vec<usize> {
        let mut subset = Vec::with_capacity(NUM_PARENTS);

        while subset.len() < NUM_PARENTS {
            let index = random_provider::random::<usize>() % limit;
            if !subset.contains(&index) {
                subset.push(index);
            }
        }

        subset.sort();
        subset
    }
}

impl<T> Alter<GraphChromosome<T>> for GraphCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    fn name(&self) -> &'static str {
        "GraphCrossover"
    }

    fn rate(&self) -> f32 {
        self.crossover_rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Alterer
    }

    #[inline]
    fn alter(
        &self,
        population: &mut Population<GraphChromosome<T>>,
        generation: i32,
    ) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;
        let mut new_phenotypes = HashMap::new();
        for index in 0..population.len() {
            if random_provider::random::<f32>() < self.crossover_rate
                && population.len() > NUM_PARENTS
            {
                let parent_indexes = GraphCrossover::<T>::distinct_subset(population.len());

                if let Some(phenotype) = self.cross(population, &parent_indexes, generation) {
                    new_phenotypes.insert(index, phenotype);
                    count += 1;
                }
            }
        }

        for (index, phenotype) in new_phenotypes.into_iter() {
            population[index] = phenotype;
        }

        let mut metric = Metric::new_operations("Graph Crossover");
        metric.add_value(count as f32);
        metric.add_duration(timer.duration());

        vec![metric]
    }
}

pub struct NodeCrossover {
    pub rate: f32,
}

impl NodeCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<T> Alter<GraphChromosome<T>> for NodeCrossover
where
    T: Clone + PartialEq + Default,
{
    fn name(&self) -> &'static str {
        "Node Crossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut GraphChromosome<T>,
        chrom_two: &mut GraphChromosome<T>,
    ) -> i32 {
        let rate = self.rate;
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                if gene_one.value.arity() != gene_two.value.arity()
                    || gene_one.node_type() != gene_two.node_type()
                {
                    continue;
                }

                let new_gene_one = gene_one.with_allele(gene_two.allele());
                let new_gene_two = gene_two.with_allele(gene_one.allele());

                chrom_one.set_gene(i, new_gene_one);
                chrom_two.set_gene(i, new_gene_two);

                cross_count += 1;
            }
        }

        cross_count
    }
}

pub struct TreeCrossover {
    pub rate: f32,
}

impl TreeCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<T> Alter<TreeChromosome<T>> for TreeCrossover
where
    T: Clone + PartialEq + Default,
{
    fn name(&self) -> &'static str {
        "Tree Crossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut TreeChromosome<T>,
        chrom_two: &mut TreeChromosome<T>,
    ) -> i32 {
        let swap_one_index = random_provider::random::<usize>() % chrom_one.len();
        let swap_two_index = random_provider::random::<usize>() % chrom_two.len();

        let one_node = &mut chrom_one.get_genes_mut()[swap_one_index];
        let two_node = &mut chrom_two.get_genes_mut()[swap_two_index];

        let one_size = one_node.size();
        let two_size = two_node.size();

        let one_rand_index = random_provider::random::<usize>() % one_size;
        let two_rand_index = random_provider::random::<usize>() % two_size;

        if one_rand_index < 1 || two_rand_index < 1 {
            return 0;
        }

        one_node.swap_subtrees(two_node, one_rand_index, two_rand_index);

        2
    }
}