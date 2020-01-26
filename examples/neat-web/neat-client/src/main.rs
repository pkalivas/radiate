
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate reqwest;

use std::net::IpAddr;
use std::fs::File;
use radiate::prelude::*;
use radiate_web::prelude::*;
use rocket::config::{Config as RConfig, Environment as REnv};
use rocket_contrib::json::{Json, JsonValue};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};



#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let data = generate_post_data();
    let local_addr = IpAddr::from([0, 0, 0, 0]);
    let client = reqwest::Client::builder()
        .local_address(local_addr)
        .build().unwrap();

    let t = reqwest::Client::new()
        .post("http://0.0.0.0:42069/")
        .json(&json!(data))
        .send()
        .await?
        .json()
        .await?;
    
    println!("{:?}", t);


    // let client = reqwest::Client::new();
    // let res = client.post("http://0.0.0.0:69/")
    //     .json(&data)
    //     .send()
    //     .await.unwrap();
    //     println!("{:?}", res);
    Ok(())
}



#[allow(dead_code)]
fn generate_post_data() -> String {
    
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
            Activation::LeakyRelu(0.02)
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
    
    // put it all together
    let radiate_dto = RadiateDto::new()
            .env(neat_env)
            .train(100, 0.3)        // this has it's own DTO too (TrainDto), but it's small
            .neat(net)
            .population(population)
            .to_json();
    
    // save to a file for testing via Postman
    radiate_dto
}