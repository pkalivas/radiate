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

use serde::{Serialize, Deserialize};

use uuid::Uuid;

use radiate::prelude::*;
use radiate_web::prelude::*;

use rocket::State;
use rocket::config::{Config as RConfig, Environment as REnv};
use rocket_contrib::json::{Json, JsonValue};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct TrainingSet {
    inputs: Vec<Vec<f32>>,
    answers: Vec<Vec<f32>>,
}

impl TrainingSet {
    pub fn new() -> Self {
        // Default to the XOR problem
        Self {
            inputs: vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![0.0, 1.0],
                vec![1.0, 1.0],
            ],
            answers: vec![
                vec![0.0],
                vec![1.0],
                vec![1.0],
                vec![0.0],
            ],
        }
    }

    pub fn new_from(training_set: Option<TrainingSetDto>) -> Self {
        training_set.map_or_else(Self::new, |set| {
            Self {
                inputs: set.inputs,
                answers: set.answers,
            }
        })
    }

    fn show(&self, model: &mut Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = model.forward(&i).unwrap();
            println!("Guess: {:.2?} Answer: {:.2}", guess, o[0]);
        }
    }
}

impl Problem<Neat> for TrainingSet {
    fn empty() -> Self { TrainingSet::new() }

    fn solve(&self, model: &mut Neat) -> f32 {
        let mut total = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        self.answers.len() as f32 - total
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SimulationStatus {
    Evolving,
    Training,
    Finished,
}

struct Simulation {
    id: Uuid,

    status: SimulationStatus,
    population: Population<Neat, NeatEnvironment, TrainingSet>,

    // generations (evolving)
    curr_gen: usize,
    num_gen: usize,

    // training
    curr_epoch: usize,
    solution: Option<Neat>,

    // Problem data.
    train: TrainDto,
    data: TrainingSet,
}

impl Simulation {
    pub fn new_from(mut radiate: RadiateDto) -> Option<Self> {
        let pop = radiate.population?;

        // take out the training variables
        let train = radiate.train?;
        let num_evolve = pop.num_evolve.unwrap_or(50);

        // set up the population now that it has been recieved
        let mut population = Population::<Neat, NeatEnvironment, TrainingSet>::new()
            .size(pop.size.unwrap_or(100))
            .configure(pop.config.unwrap_or_else(|| Config::new()))
            .debug(pop.debug_process.unwrap_or(false))
            .dynamic_distance(pop.dynamic_distance.unwrap_or(false));

        let data = if let Some(set) = radiate.training_set.take() {
            let data = TrainingSet::new_from(Some(set));
            // TODO: Remove this once we have a custom `run()` loop
            population = population.impose(data.clone());
            data
        } else {
            TrainingSet::new()
        };
        if let Some(env) = radiate.env {
            population = population.constrain(env);
        }
        if let Some(net) = radiate.neat {
            population = population.populate_clone(net);
        }
        if let Some(stagnation) = pop.stagnation {
            if let Some(genocide) = pop.genocide {
                population = population.stagnation(stagnation, genocide);
            }
        }
        if let Some(parental_criteria) = pop.parental_criteria {
            population = population.parental_criteria(parental_criteria);
        }
        if let Some(survivor_criteria) = pop.survivor_criteria {
            population = population.survivor_criteria(survivor_criteria);
        }

        Some(Self {
            id: Uuid::new_v4(),
            status: SimulationStatus::Evolving,
            population,

            curr_gen: 0,
            num_gen: num_evolve as usize,

            curr_epoch: 0,
            solution: None,

            train,
            data,
        })
    }

    pub fn get_status(&self) -> JsonValue {
        json!({
          "status": self.status,
          "curr_gen": self.curr_gen,
          "curr_epoch": self.curr_epoch,
        })
    }

    fn step_gen(&mut self) {
        // TODO: queue this work for a worker to run.
        if self.curr_gen >= self.num_gen {
            return;
        }
        self.curr_gen += 1;
        match self.population.train() {
            Some((fit , top)) => {
                println!("epoch: {} score: {}", self.curr_gen, fit);
                if self.curr_gen == self.num_gen {
                    self.solution = Some(top);
                    self.status = SimulationStatus::Training;
                }
            },
            None => {
            },
        }
    }

    fn step_train(&mut self) {
        // TODO: queue this work for a worker to run.
        if self.curr_epoch >= self.train.epochs as usize {
            return;
        }
        // manually train the neural net
        let num_train = self.train.epochs as usize;
        let learning_rate = self.train.learning_rate;
        if let Some(solution) = &mut self.solution {
            solution.train(&self.data.inputs, &self.data.answers, learning_rate, Loss::Diff, |iter, _| {
                iter == num_train
            }).unwrap();

            // show it
            self.data.show(solution);
        }

        self.status = SimulationStatus::Finished;
        self.curr_epoch = num_train;
    }

    pub fn run(&mut self) {
        match self.status {
            SimulationStatus::Evolving => {
                self.step_gen();
            },
            SimulationStatus::Training => {
                self.step_train();
            },
            _ => (),
        }
    }
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
        let id = sim.id;
        self.simulations.write().unwrap().insert(id, Arc::new(RwLock::new(sim)));
        Some(id)
    }
}


fn main() {
    // generate_post_data();
    let r_config = RConfig::build(REnv::Staging)
        .address("0.0.0.0")
        .port(42069)
        .finalize()
        .unwrap();

    rocket::custom(r_config)
        .mount("/", routes![get_sim_status, new_sim])
        .manage(SimStorage::new())
        .launch();
}



#[get("/simulations/<id>")]
fn get_sim_status(sims: State<SimStorage>, id: String) -> Option<JsonValue> {
    if let Some(sim) = sims.get(&id) {
        let mut sim = sim.write().unwrap();
        // fake workers
        sim.run();
        let res = Some(sim.get_status());
        res
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

