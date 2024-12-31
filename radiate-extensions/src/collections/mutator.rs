use super::{Graph, GraphChromosome, GraphNode, NodeType};

use radiate::{random_provider, timer::Timer, Alter, AlterType, Chromosome, Metric, Population};

use std::sync::Arc;

use crate::ops::operation::Operation;

use radiate::engines::genome::genes::gene::Gene;

pub enum NodeMutate {
    Forward(NodeType, f32),
    Recurrent(NodeType, f32),
}

impl NodeMutate {
    pub fn node_type(&self) -> NodeType {
        match self {
            NodeMutate::Forward(node_type, _) => *node_type,
            NodeMutate::Recurrent(node_type, _) => *node_type,
        }
    }

    pub fn rate(&self) -> f32 {
        match self {
            NodeMutate::Forward(_, rate) => *rate,
            NodeMutate::Recurrent(_, rate) => *rate,
        }
    }

    pub fn is_recurrent(&self) -> bool {
        match self {
            NodeMutate::Forward(_, _) => false,
            NodeMutate::Recurrent(_, _) => true,
        }
    }
}

pub struct GraphMutator {
    mutations: Vec<NodeMutate>,
}

impl GraphMutator {
    pub fn new(mutations: Vec<NodeMutate>) -> Self {
        Self { mutations }
    }
}

impl<T> Alter<GraphChromosome<T>> for GraphMutator
where
    T: Clone + PartialEq + Default,
{
    fn name(&self) -> &'static str {
        "GraphMutator"
    }

    fn rate(&self) -> f32 {
        1.0
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
        for i in 0..population.len() {
            let phenotype = &mut population[i];
            let genotype = &mut phenotype.genotype();

            let chromosome_index = random_provider::random::<usize>() % genotype.len();

            let chromosome = &mut phenotype.genotype_mut()[chromosome_index];

            if self.mutate_chromosome(chromosome) > 0 {
                count += 1;
                phenotype.set_score(None);
                phenotype.generation = generation;
            }
        }

        let mut result = Metric::new_operations("GraphMutator");
        result.add_value(count as f32);
        result.add_duration(timer.duration());

        vec![result]
    }

    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<T>) -> i32 {
        let mutation = random_provider::choose(&self.mutations);

        if random_provider::random::<f32>() > mutation.rate() {
            return 0;
        }

        if let Some(ref factory) = chromosome.factory {
            let mut graph = Graph::new(chromosome.nodes.clone());
            let node_fact = factory.borrow();

            if self.add_node(
                &mut graph,
                &mutation.node_type(),
                &node_fact,
                mutation.is_recurrent(),
            ) {
                chromosome.nodes = graph.into_iter().collect::<Vec<GraphNode<T>>>();
                return 1;
            }
        }

        0
    }
}

pub struct OperationMutator {
    pub rate: f32,
    pub replace_rate: f32,
}

impl OperationMutator {
    pub fn new(rate: f32, replace_rate: f32) -> Self {
        Self { rate, replace_rate }
    }
}

impl<T> Alter<GraphChromosome<T>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn name(&self) -> &'static str {
        "OpMutator"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Mutator
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<T>) -> i32 {
        let mutation_indexes = (0..chromosome.len())
            .filter(|_| random_provider::random::<f32>() < self.rate)
            .collect::<Vec<usize>>();

        if mutation_indexes.is_empty() {
            return 0;
        }

        for &i in mutation_indexes.iter() {
            let curreent_node = chromosome.get_gene(i);

            match curreent_node.allele() {
                Operation::MutableConst {
                    name,
                    arity,
                    value,
                    get_value,
                    modifier,
                    operation,
                } => {
                    let new_value = get_value();
                    let modified_value = modifier(value);

                    let new_op = Operation::MutableConst {
                        name,
                        arity: *arity,
                        value: if random_provider::random::<f32>() < self.replace_rate {
                            new_value
                        } else {
                            modified_value
                        },
                        modifier: Arc::clone(modifier),
                        get_value: Arc::clone(get_value),
                        operation: Arc::clone(operation),
                    };

                    chromosome.set_gene(i, curreent_node.with_allele(&new_op));
                }
                _ => {}
            }
        }

        mutation_indexes.len() as i32
    }
}
