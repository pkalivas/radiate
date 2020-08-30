
// Bring everything in the lib into scope

pub mod prelude;
pub mod models;
pub mod engine;

#[macro_use]
extern crate serde_derive;

pub use models::{
    neat::{
        layers::{
            layer::Layer,
            layertype::LayerType,
            dense::Dense,
            lstm::LSTM,
            gru::GRU
        },
        neurontype::NeuronType,
        loss::Loss,
        edge::Edge,
        neuron::Neuron,
        neat::Neat,
        neatenv::NeatEnvironment,
        activation::Activation,
    }
};


pub use engine::{
    population::*,
    genome::Genome,
    problem::Problem,
    niche::Niche,
    generation::*,
    genocide::Genocide,
    environment::Envionment,
    survival::SurvivalCriteria,
    survival::ParentalCriteria
};


/// Default environment for the NEAT algorithm as described in the paper.
///
/// These are very basic and are used to solve the xor problem for NEAT.
pub fn default_neat_env() -> NeatEnvironment {
    NeatEnvironment::new()
        .set_input_size(3)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.5)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.2)
        .set_activation_functions(vec![Activation::Sigmoid])
}
