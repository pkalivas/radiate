#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use std::collections::HashMap;
use std::sync::{RwLock, Arc};

use uuid::Uuid;

use radiate_web::prelude::*;

use rocket::State;
use rocket::config::{Config as RConfig, Environment as REnv};
use rocket_contrib::json::{Json, JsonValue};

use neat_server::*;

#[derive(Default)]
struct SimStorage {
    simulations: RwLock<HashMap<Uuid, Arc<RwLock<Simulation>>>>,
}

impl SimStorage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, id: &str) -> Option<Arc<RwLock<Simulation>>> {
        let id = Uuid::parse_str(id).ok()?;
        self.simulations.read().unwrap().get(&id).map(|v| v.clone())
    }

    pub fn add(&self, sim: Simulation) -> Option<Uuid> {
        let id = sim.id();
        self.simulations.write().unwrap().insert(id, Arc::new(RwLock::new(sim)));
        Some(id)
    }

    fn encode_work(&self, sim: &mut Simulation) -> JsonValue {
        if let Some(work) = sim.get_work() {
            if let Some(member) = sim.work_member(&work) {
                return json!({
                    "work": Some((work, member)),
                });
            } else {
                unreachable!("This shouldn't happen.");
            }
        }
        json!({})
    }

    pub fn get_work(&self) -> JsonValue {
        for sim in self.simulations.read().unwrap().values() {
            let mut sim = sim.write().unwrap();
            return self.encode_work(&mut sim);
        }
        json!({})
    }

    pub fn sim_get_work(&self, id: &str) -> JsonValue {
        let sim = self.get(&id);
        if let Some(sim) = sim {
            let mut sim = sim.write().unwrap();
            self.encode_work(&mut sim)
        } else {
            json!({})
        }
    }
}


fn main() {
    env_logger::init();
    // generate_post_data();
    let r_config = RConfig::build(REnv::Production)
        .address("0.0.0.0")
        .port(42069)
        .finalize()
        .unwrap();

    rocket::custom(r_config)
        .mount("/", routes![
            get_work,
            get_sim_status,
            get_sim_training_set,
            sim_get_work,
            update_work,
            new_sim,
        ])
        .manage(SimStorage::new())
        .launch();
}


#[get("/get_work")]
fn get_work(sims: State<SimStorage>) -> JsonValue {
    sims.get_work()
}

#[get("/simulations/<id>/get_work")]
fn sim_get_work(sims: State<SimStorage>, id: String) -> JsonValue {
    sims.sim_get_work(&id)
}

#[get("/simulations/<id>")]
fn get_sim_status(sims: State<SimStorage>, id: String) -> Option<JsonValue> {
    if let Some(sim) = sims.get(&id) {
        let sim = sim.read().unwrap();
        //let mut sim = sim.write().unwrap();
        // TODO: remove
        //sim.run();
        Some(json!(sim.get_status()))
    } else {
        None
    }
}

#[get("/simulations/<id>/training_set")]
fn get_sim_training_set(sims: State<SimStorage>, id: String) -> Option<JsonValue> {
    if let Some(sim) = sims.get(&id) {
        let sim = sim.read().unwrap();
        Some(json!(sim.get_training_set()))
    } else {
        None
    }
}

#[post("/simulations/<id>/update_work", format = "json", data = "<result>")]
fn update_work(sims: State<SimStorage>, id: String, result: Json<WorkResult>) -> Option<JsonValue> {
    if let Some(sim) = sims.get(&id) {
        let mut sim = sim.write().unwrap();
        sim.update_work(result.0);
        Some(json!(sim.get_status()))
    } else {
        None
    }
}

#[post("/simulations", format = "json", data = "<radiate>")]
fn new_sim(sims: State<SimStorage>, radiate: Json<RadiateDto>) -> Option<JsonValue> {
    let sim = Simulation::new_from(radiate.0)?;
    let id = sims.add(sim)?;
    Some(json!({ "id": format!("{}", id.to_hyphenated()) }))
}

