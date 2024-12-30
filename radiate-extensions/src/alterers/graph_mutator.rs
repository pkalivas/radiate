use crate::collections::*;

use radiate::alter::AlterType;
use radiate::engines::alterers::Alter;
use radiate::engines::genome::*;
use radiate::timer::Timer;
use radiate::{random_provider, Metric};

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

pub struct GraphMutator<T>
where
    T: Clone + PartialEq + Default,
{
    pub mutations: Vec<NodeMutate>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GraphMutator<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(mutations: Vec<NodeMutate>) -> Self {
        Self {
            mutations,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Alter<GraphChromosome<T>> for GraphMutator<T>
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

        let mut result = Metric::new_operations(self.name());
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
