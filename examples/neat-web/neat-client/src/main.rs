extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate reqwest;

use std::time::Duration;

use uuid::Uuid;

use env_logger;

use serde::Deserialize;

use radiate::prelude::*;
use radiate_web::prelude::*;

use tokio::time::delay_for;
 
#[derive(Debug, Default, Deserialize)]
struct SimStatus {
    status: String,
    curr_gen: usize,
    curr_fitness: Option<f32>,
    last_gen_elapsed: Option<Duration>,
    solution: Option<Neat>,
}

#[derive(Debug, Default, Deserialize)]
struct AddSim {
    id: Uuid,
}

async fn get_sim_status(url: &str) -> Result<SimStatus, reqwest::Error> {
    // Reqwest doesn't seem to handle "Connection: close" correctly.
    // Don't re-use client.
    let client = reqwest::Client::new();
    let status = client.get(url)
        .send().await?
        .json::<SimStatus>().await?;
    println!("sim_status = {:?}, gen = {}, fit = {:?}, elapsed = {:?}",
      status.status, status.curr_gen, status.curr_fitness, status.last_gen_elapsed);

    Ok(status)
}

async fn new_sim(base_url: &str, data: &RadiateDto) -> Result<Uuid, reqwest::Error> {
    let client = reqwest::Client::new();
    let sim = client.post(base_url)
        .json(data)
        .send().await?
        .json::<AddSim>().await?;
    println!("sim = {:?}", sim);

    Ok(sim.id)
}

fn sim_finished(simulation: &RadiateDto, mut status: SimStatus) {
    if let Some(solution) = status.solution.as_mut() {
        println!("solution = {:?}", solution);
        if let Some(data) = &simulation.training_set {
            println!();
            for (i, o) in data.inputs.iter().zip(data.answers.iter()) {
                let guess = solution.forward(&i).unwrap();
                println!("Guess: {:.2?} Answer: {:.2}", guess, o[0]);
            }
        }
    } else {
        println!("No solution returned from simulation");
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    env_logger::init();
    let base_url = "http://0.0.0.0:42069/simulations";

    let name = std::env::args().nth(1).unwrap_or("XOR".to_string());
    let sim_id = std::env::args().nth(2);

    let simulation = build_simulation(&name)
        .expect("unknown problem");

    let sim_id = match sim_id {
        Some(id) => {
          println!("Check existing simulation: id={}", id);
          id
        },
        None => {
          println!("Create new simulation:");
          new_sim(base_url, &simulation).await?.to_string()
        }
    };

    let status_url = format!("{}/{}", base_url, sim_id);

    loop {
        let status = get_sim_status(&status_url).await?;
        if status.status == "Finished" {
            sim_finished(&simulation, status);
            break;
        }
        delay_for(Duration::from_millis(500)).await;
    }

    Ok(())
}

fn build_simulation(name: &str) -> Option<RadiateDto> {
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

    // build the population
    let population = NeatPopulationBuilder::new()
            .num_evolve(500)
            .target_fitness(3.8)
            .size(300)
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

    // Build network, inputs, answers for the named problem.
    let (net, inputs, answers) = match name.to_uppercase().as_str() {
      "XOR" => {
          // build the neat network
          let net = Neat::new()
              .input_size(2)
              .batch_size(1)
              .dense(7, Activation::Relu)
              .dense_pool(1, Activation::Sigmoid);

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

          (net, inputs, answers)
      },
      // TODO: add more simulations
      _ => {
          return None;
      }
    };

    // put it all together
    let radiate_dto = RadiateDto::new()
            .env(neat_env)
            .train(200, 0.15)        // this has it's own DTO too (TrainDto), but it's small
            .training_set(inputs, answers)
            .neat(net)
            .population(population);

    Some(radiate_dto)
}
