#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use std::error::Error;
use radiate::prelude::*;
use radiate_web::prelude::*;
use serde::{Serialize, Deserialize};
use rocket_contrib::json::{Json, JsonValue};




fn main() {
    rocket::ignite()
        .mount("/", routes![run])
        .launch();
}



#[post("/", format = "json", data = "<env>")]
fn run(env: Json<RadiateDto>) -> JsonValue {
    println!("Recieved: \n{:#?}", env.0);
    json!({"it": "works"})
}


// fn main() -> Result<(), Box<dyn Error>> {

//     let mut neat_env = NeatEnvironment::new()
//         .set_input_size(2)
//         .set_output_size(1)
//         .set_weight_mutate_rate(0.8)
//         .set_edit_weights(0.1)
//         .set_weight_perturb(1.75)
//         .set_new_node_rate(0.03)
//         .set_new_edge_rate(0.04)
//         .set_reactivate(0.2)
//         .set_activation_functions(vec![
//             Activation::LeakyRelu(0.02)
//         ]);

//     let net = Neat::new()
//         .input_size(1)
//         .batch_size(1)
//         .dense_pool(3, Activation::Sigmoid)
//         .dense(1, Activation::Sigmoid);

//     let population = NeatPopulationBuilder::new()
//             .size(100)
//             .dynamic_distance(true)
//             .debug_process(true)
//             .config(Config {
//                 inbreed_rate: 0.001,
//                 crossover_rate: 0.75,
//                 distance: 0.5,
//                 species_target: 5
//             })
//             .stagnation(10)
//             .genocide(vec![Genocide::KillWorst(0.9)]);
    
//     let radiate_json = RadiateDto::new()
//             .env(neat_env)
//             .train(100, 0.3)
//             .neat(net)
//             .population(population)
//             .to_json();

//             println!("{:#?}", radiate_json);

//     Ok(())
// }



