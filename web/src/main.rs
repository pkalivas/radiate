
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate radiate;

use serde::{Serialize, Deserialize};
use rocket_contrib::json::{Json, JsonValue};
use radiate::prelude::*;


fn main() {
    rocket::ignite()
        .mount("/", routes![run])
        .launch();
}



#[post("/", format = "Application/json", data = "<env>")]
fn run(env: Json<NeatEnvDto>) -> JsonValue {
    println!("Recieved: \n{:#?}", env.0.to_env());
    json!({"it": "works"})
}



// data
// {
//     "weight_mutate_rate": 0.8,
//     "weight_perturb": 0.4,
//     "new_node_rate": 0.4,
//     "new_edge_rate": 0.3,
//     "edit_weights": 0.5,
//     "reactivate": 0.2,
//     "input_size": 1,
//     "output_size": 2,
//     "activation_functions": [
//     	{
//     		"activation" : 1,
//     		"value" : 0
//     	},
//     	{
//     		"activation": 5,
//     		"value": 5
//     	}
//     ]
// }