// Bring everything in the lib into scope

pub mod engine;
pub mod models;
pub mod prelude;

#[macro_use]
extern crate serde_derive;

pub use models::neat::{
    activation::Activation,
    edge::Edge,
    layers::{dense::Dense, gru::GRU, layer::Layer, layertype::LayerType, lstm::LSTM},
    loss::Loss,
    neat::Neat,
    neatenv::NeatEnvironment,
    neuron::Neuron,
    neurontype::NeuronType,
};

pub use engine::{
    environment::Envionment, generation::*, genocide::Genocide, genome::Genome, niche::Niche,
    population::*, problem::Problem, survival::ParentalCriteria, survival::SurvivalCriteria,
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
