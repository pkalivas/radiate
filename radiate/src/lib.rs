
/// Bring everything in the lib into scope

pub mod prelude;
pub mod models;
pub mod engine;

pub use models::{
    evtree::{
        evtree::Evtree,
        node::Node,
        network::NeuralNetwork,
        evenv::TreeEnvionment
    },
    neat::{
        layer::Layer,
        nodetype::NodeType,
        edge::Edge,
        vertex::Vertex,
        counter::Counter,
        neat::Neat,
        neatenv::NeatEnvironment,
        neurons::{
            neuron::Neuron,
            activation::Activation,
            dense::Dense
        }
    }
};

pub use engine::{
    population::*,
    genome::Genome,
    problem::Problem,
    niche::Niche,
    generation::Generation,
    genocide::Genocide,
    environment::Envionment,
    survival::SurvivalCriteria,
    survival::ParentalCriteria
};


/// get a default environment settings for evtree, these are very basic and 
/// are used to solve the xor problem. These are just settings to evolve thre
/// tree, more can be added or taken away depending on the desired problem to solve
pub fn defualt_evtree_env() -> TreeEnvionment {
    TreeEnvionment::new()
        .set_input_size(2)
        .set_outputs(vec![0, 1])
        .set_start_height(2)
        .set_max_height(3)
        .set_network_mutation_rate(0.5)
        .set_node_add_rate(0.1)
        .set_gut_rate(0.05)
        .set_shuffle_rate(0.05)
        .set_layer_mutate_rate(0.8)
        .set_weight_mutate_rate(0.8)
        .set_weight_transform_rate(5.0)
}
/// Defualt enviornment for the neat algorithm as desribed in the papaer - like evtree
/// these are very basic and are used to solve the xor problem for neat
pub fn default_neat_env() -> NeatEnvironment {
    NeatEnvironment::new()
        .set_input_size(3)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(2.0)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.25)
        .set_c1(1.0)
        .set_c2(1.0)
        .set_c3(0.4)
        .set_activation_functions(vec![Activation::Sigmoid])
        .start_innov_counter()
}
