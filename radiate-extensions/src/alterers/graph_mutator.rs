use crate::architects::node_collections::*;
use crate::architects::schema::node_types::NodeType;
use crate::node::Node;
use crate::schema::collection_type::CollectionType;
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
    _marker: std::marker::PhantomData<T>,
}

impl<T> GraphMutator<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub fn new(mutations: Vec<NodeMutate>) -> Self {
        Self {
            mutations,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn insert_forward_node(
        &self,
        collection: &[Node<T>],
        node_type: &NodeType,
        factory: &NodeFactory<T>,
    ) -> Option<Vec<Node<T>>> {
        let source_node = random_source_node(collection);
        let target_node = random_target_node(collection);
        let source_node_index = source_node.index;
        let target_node_index = target_node.index;

        let new_source_edge_index = collection.len();
        let new_node_index = collection.len() + 1;
        let new_target_edge_index = collection.len() + 2;

        if source_node.node_type == NodeType::Weight && node_type != &NodeType::Weight {
            let incoming_node = collection
                .get(*source_node.incoming.iter().next().unwrap())
                .unwrap();
            let outgoing_node = collection
                .get(*source_node.outgoing.iter().next().unwrap())
                .unwrap();

            let new_source_edge = factory.new_node(new_source_edge_index, source_node.node_type);
            let new_node = factory.new_node(new_node_index, *node_type);
            let new_target_edge = factory.new_node(new_target_edge_index, source_node.node_type);

            if is_locked(outgoing_node) {
                let mut temp = Graph::from_nodes(
                    collection
                        .iter()
                        .cloned()
                        .chain(vec![new_source_edge, new_node])
                        .collect::<Vec<Node<T>>>(),
                );

                temp.attach(source_node_index, new_node_index);
                temp.attach(new_node_index, new_source_edge_index);
                temp.attach(new_source_edge_index, outgoing_node.index);
                temp.detach(source_node_index, outgoing_node.index);

                return self.repair_insert(
                    temp,
                    new_node_index,
                    incoming_node,
                    outgoing_node,
                    false,
                );
            } else {
                let mut temp = Graph::from_nodes(
                    collection
                        .iter()
                        .cloned()
                        .chain(vec![new_source_edge, new_node, new_target_edge])
                        .collect::<Vec<Node<T>>>(),
                );

                temp.attach(source_node.index, new_source_edge_index);
                temp.attach(new_source_edge_index, new_node_index);
                temp.attach(new_node_index, new_target_edge_index);
                temp.attach(new_target_edge_index, outgoing_node.index);

                return self.repair_insert(
                    temp,
                    new_node_index,
                    incoming_node,
                    outgoing_node,
                    false,
                );
            }
        } else if !can_connect(collection, source_node.index, target_node.index, false) {
            return None;
        }

        let mut temp = Graph::from_nodes(
            collection
                .iter()
                .cloned()
                .chain(vec![factory.new_node(collection.len(), *node_type)])
                .collect::<Vec<Node<T>>>(),
        );

        temp.attach(source_node_index, collection.len());
        temp.attach(collection.len(), target_node_index);
        temp.detach(source_node_index, target_node_index);

        self.repair_insert(temp, collection.len(), source_node, target_node, false)
    }

    #[inline]
    pub fn insert_recurrent_node(
        &self,
        collection: &[Node<T>],
        node_type: &NodeType,
        factory: &NodeFactory<T>,
    ) -> Option<Vec<Node<T>>> {
        let source_node = random_source_node(collection);
        let target_node = random_target_node(collection);
        let source_node_index = source_node.index;
        let target_node_index = target_node.index;

        let new_source_edge_index = collection.len();
        let new_node_index = collection.len() + 1;
        let new_target_edge_index = collection.len() + 2;
        let recurrent_edge_index = collection.len() + 3;

        if source_node.node_type == NodeType::Weight && node_type != &NodeType::Weight {
            let incoming_node = collection
                .get(*source_node.incoming.iter().next().unwrap())
                .unwrap();
            let outgoing_node = collection
                .get(*source_node.outgoing.iter().next().unwrap())
                .unwrap();

            let new_source_edge = factory.new_node(new_source_edge_index, source_node.node_type);
            let new_node = factory.new_node(new_node_index, *node_type);
            let new_target_edge = factory.new_node(new_target_edge_index, source_node.node_type);
            let recurrent_edge = factory.new_node(recurrent_edge_index, source_node.node_type);

            return if is_locked(outgoing_node) {
                let mut temp = Graph::from_nodes(
                    collection
                        .iter()
                        .cloned()
                        .chain(vec![new_source_edge, new_node, new_target_edge])
                        .collect::<Vec<Node<T>>>(),
                );

                temp.attach(incoming_node.index, new_node_index);
                temp.attach(new_node_index, new_source_edge_index);
                temp.attach(new_source_edge_index, new_node_index);
                temp.attach(new_node_index, new_target_edge_index);
                temp.attach(new_target_edge_index, outgoing_node.index);
                temp.detach(incoming_node.index, outgoing_node.index);

                self.repair_insert(temp, new_node_index, incoming_node, outgoing_node, true)
            } else if !source_node.is_recurrent() {
                let mut temp = Graph::from_nodes(
                    collection
                        .iter()
                        .cloned()
                        .chain(vec![
                            new_source_edge,
                            new_node,
                            new_target_edge,
                            recurrent_edge,
                        ])
                        .collect::<Vec<Node<T>>>(),
                );

                temp.attach(incoming_node.index, new_source_edge_index);
                temp.attach(new_source_edge_index, new_node_index);
                temp.attach(new_node_index, new_target_edge_index);
                temp.attach(new_target_edge_index, outgoing_node.index);
                temp.attach(recurrent_edge_index, new_node_index);
                temp.attach(new_node_index, recurrent_edge_index);

                self.repair_insert(temp, new_node_index, incoming_node, outgoing_node, true)
            } else {
                let mut temp = Graph::from_nodes(
                    collection
                        .iter()
                        .cloned()
                        .chain(vec![new_source_edge, new_node, new_target_edge])
                        .collect::<Vec<Node<T>>>(),
                );

                temp.attach(incoming_node.index, new_source_edge_index);
                temp.attach(new_source_edge_index, new_node_index);
                temp.attach(new_node_index, new_target_edge_index);
                temp.attach(new_target_edge_index, outgoing_node.index);

                self.repair_insert(temp, new_node_index, incoming_node, outgoing_node, true)
            };
        } else if !can_connect(collection, source_node.index, target_node.index, true) {
            return None;
        }

        let mut temp = Graph::from_nodes(
            collection
                .iter()
                .cloned()
                .chain(vec![factory.new_node(collection.len(), *node_type)])
                .collect::<Vec<Node<T>>>(),
        );

        temp.attach(source_node_index, collection.len());
        temp.attach(collection.len(), target_node_index);
        temp.detach(source_node_index, target_node_index);

        self.repair_insert(temp, collection.len(), source_node, target_node, true)
    }

    #[inline]
    fn repair_insert(
        &self,
        mut collection: Graph<T>,
        new_node_index: usize,
        source_node: &Node<T>,
        target_node: &Node<T>,
        recurrent: bool,
    ) -> Option<Vec<Node<T>>> {
        for _ in 0..collection.get(new_node_index).value.arity() - 1 {
            let other_source_node = random_source_node(collection.get_nodes());
            if can_connect(
                collection.get_nodes(),
                other_source_node.index,
                new_node_index,
                recurrent,
            ) {
                collection.attach(other_source_node.index, new_node_index);
            }
        }

        for node in collection.iter_mut() {
            node.collection_type = Some(CollectionType::Graph);
        }

        if !collection.is_valid() {
            return None;
        }

        Some(
            collection
                .set_cycles(vec![source_node.index, target_node.index])
                .into_iter()
                .collect::<Vec<Node<T>>>(),
        )
    }
}

impl<T> Alter<NodeChromosome<T>> for GraphMutator<T>
where
    T: Clone + PartialEq + Default + 'static,
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
        population: &mut Population<NodeChromosome<T>>,
        generation: i32,
    ) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;
        for i in 0..population.len() {
            let mutation = random_provider::choose(&self.mutations);

            if random_provider::random::<f32>() > mutation.rate() {
                continue;
            }

            let genotype = population[i].genotype();
            let chromosome_index = random_provider::random::<usize>() % genotype.len();
            let chromosome = &genotype[chromosome_index];

            if let Some(ref factory) = chromosome.factory {
                let mutated_graph = if mutation.is_recurrent() {
                    let node_fact = factory.borrow();
                    self.insert_recurrent_node(&chromosome.nodes, &mutation.node_type(), &node_fact)
                } else {
                    let node_fact = factory.borrow();
                    self.insert_forward_node(&chromosome.nodes, &mutation.node_type(), &node_fact)
                };

                if let Some(mutated_graph) = mutated_graph {
                    if !mutated_graph.iter().all(|node| node.is_valid()) {
                        continue;
                    }

                    if mutated_graph.len() == chromosome.nodes.len() {
                        continue;
                    }

                    let mut copied_genotype = genotype.clone();

                    count += 1;

                    copied_genotype[chromosome_index] =
                        NodeChromosome::with_factory(mutated_graph, factory.clone());
                    population[i] = Phenotype::from_genotype(copied_genotype, generation);
                }
            }
        }

        let mut result = Metric::new_operations(self.name());
        result.add_value(count as f32);
        result.add_duration(timer.duration());

        vec![result]
    }
}
