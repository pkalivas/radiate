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

fn work_to_json(work: Option<WorkUnit>) -> JsonValue {
    json!(GetWorkResp {
        work: work,
    })
}

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
        println!("Add new simulation: {}", id);
        self.simulations.write().unwrap().insert(id, Arc::new(RwLock::new(sim)));
        Some(id)
    }

    pub fn get_work(&self, id: Option<&str>) -> JsonValue {
        if let Some(id) = id {
            // check simulation for more queued work
            let sim = self.get(&id);
            if let Some(sim) = sim {
                let mut sim = sim.write().unwrap();
                return work_to_json(sim.get_work());
            }
        } else {
            // find simulation with queued work.
            for sim in self.simulations.read().unwrap().values() {
                let mut sim = sim.write().unwrap();
                if let Some(work) = sim.get_work() {
                    return work_to_json(Some(work));
                }
            }
        }
        work_to_json(None)
    }

    pub fn work_results(&self, id: &str, result: GetWorkResult, get_work: bool) -> Option<JsonValue> {
        if let Some(sim) = self.get(&id) {
            let mut sim = sim.write().unwrap();
            sim.work_results(result);
            if get_work {
                Some(work_to_json(sim.get_work()))
            } else {
                Some(work_to_json(None))
            }
        } else {
            None
        }
    }
}


fn main() {
    env_logger::init();
    // generate_post_data();
    let r_config = RConfig::build(REnv::Production)
        .address("0.0.0.0")
        .port(42069)
        .keep_alive(0) // Rocket's keep-alive is broken.  Disable for now.
        .finalize()
        .unwrap();

    rocket::custom(r_config)
        .mount("/", routes![
            get_work,
            get_sim_status,
            get_sim_training_set,
            get_solution,
            sim_get_work,
            work_results,
            new_sim,
        ])
        .manage(SimStorage::new())
        .launch();
}


#[get("/get_work")]
fn get_work(sims: State<SimStorage>) -> JsonValue {
    sims.get_work(None)
}

#[get("/simulations/<id>/get_work")]
fn sim_get_work(sims: State<SimStorage>, id: String) -> JsonValue {
    sims.get_work(Some(&id))
}

#[get("/simulations/<id>")]
fn get_sim_status(sims: State<SimStorage>, id: String) -> Option<JsonValue> {
    if let Some(sim) = sims.get(&id) {
        let sim = sim.read().unwrap();
        Some(json!(sim.get_status()))
    } else {
        None
    }
}

#[get("/simulations/<id>/solution")]
fn get_solution(sims: State<SimStorage>, id: String) -> Option<JsonValue> {
    if let Some(sim) = sims.get(&id) {
        let sim = sim.read().unwrap();
        Some(json!(sim.get_solution()))
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

#[post("/simulations/<id>/work_results?<get_work>", format = "json", data = "<result>")]
fn work_results(sims: State<SimStorage>, id: String, get_work: bool, result: Json<GetWorkResult>) -> Option<JsonValue> {
    sims.work_results(&id, result.0, get_work)
}

#[post("/simulations", format = "json", data = "<radiate>")]
fn new_sim(sims: State<SimStorage>, radiate: Json<RadiateDto>) -> Option<JsonValue> {
    let sim = Simulation::new_from(radiate.0)?;
    let id = sims.add(sim)?;
    Some(json!({ "id": format!("{}", id.to_hyphenated()) }))
}

