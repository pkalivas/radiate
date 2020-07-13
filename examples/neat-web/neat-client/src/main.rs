#![feature(proc_macro_hygiene, decl_macro)]

extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate reqwest;

use env_logger;

use serde::Deserialize;

use radiate::prelude::*;
use radiate_web::prelude::*;

use reqwest::Client;
 
#[derive(Debug, Default, Deserialize)]
struct SimStatus {
    status: String,
    curr_gen: usize,
}

#[derive(Debug, Default, Deserialize)]
struct AddSim {
    id: String,
}

async fn get_sim_status(client: &Client, url: &str) -> Result<SimStatus, reqwest::Error> {
    let status = client.get(url)
        .send().await?
        .json::<SimStatus>().await?;
    println!("sim_status = {:?}", status);

    Ok(status)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    env_logger::init();

    let data = generate_post_data();

    let base_url = "http://0.0.0.0:42069/simulations";
    let client = reqwest::Client::new();
    let add_sim = client.post(base_url)
        .json(&data)
        .send().await?
        .json::<AddSim>().await?;
    println!("add_sim = {:?}", add_sim);

    let status_url = format!("{}/{}", base_url, add_sim.id);

    loop {
        let status = get_sim_status(&client, &status_url).await?;
        if status.status == "Finished" {
            break;
        }
    }

    Ok(())
}

fn generate_post_data() -> RadiateDto {
    // create an environment
    let neat_env = NeatEnvironment::new()
        .set_input_size(2)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.75)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.2)
        .set_activation_functions(vec![
            Activation::Relu,
            Activation::Sigmoid
        ]);

    // build the neat network
    let net = Neat::new()
        .input_size(2)
        .batch_size(1)
        .dense_pool(1, Activation::Sigmoid);

    // build the population
    let population = NeatPopulationBuilder::new()
            .num_evolve(100)
            .size(100)
            .dynamic_distance(true)
            .debug_process(true)
            .config(Config {
                inbreed_rate: 0.001,
                crossover_rate: 0.75,
                distance: 0.5,
                species_target: 5
            })
            .stagnation(10)
            .genocide(vec![Genocide::KillWorst(0.9)]);
    
    let inputs = vec![
        vec![0.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
    ];
    let answers = vec![
        vec![0.0],
        vec![0.0],
        vec![1.0],
        vec![1.0],
    ];
    // put it all together
    let radiate_dto = RadiateDto::new()
            .env(neat_env)
            .train(100, 0.3)        // this has it's own DTO too (TrainDto), but it's small
            .training_set(inputs, answers)
            .neat(net)
            .population(population);

    radiate_dto
}
