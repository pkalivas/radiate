#![feature(proc_macro_hygiene, decl_macro)]

extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate reqwest;

use std::time::Duration;
use std::io::{self, Write};

use std::collections::{HashMap, hash_map::Entry};

use uuid::Uuid;

use env_logger;

use tokio::time::delay_for;
 
use radiate_web::*;
use radiate::prelude::*;
use neat_server::*;

fn flush() {
  io::stdout().flush().ok().expect("Failed to flush stdout")
}

#[derive(Debug, Default)]
struct CacheSimData {
    training_data: HashMap<Uuid, TrainingSet>,
}

impl CacheSimData {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn get_sim_data(&mut self, base_url: &str, id: Uuid) -> Result<&TrainingSet, reqwest::Error> {
        let entry = self.training_data.entry(id);
        // check cache.
        match entry {
            Entry::Occupied(val) => {
                // got cached value.
                Ok(val.into_mut())
            },
            Entry::Vacant(val) => {
                let url = format!("{}/simulations/{}/training_set", base_url, id);
                // Work around reqwest issue with "Connection: close", don't re-use client.
                let client = reqwest::Client::new();
                let data = client.get(&url)
                  .send().await?
                  .json::<TrainingSet>().await?;
                Ok(val.insert(data))
            },
        }
    }
}

async fn work_results(base_url: &str, id: Uuid, work: &mut WorkUnit, fitness: Option<f32>) -> Result<Option<WorkUnit>, reqwest::Error> {
    let result = GetWorkResult {
        id: work.id,
        curr_gen: work.curr_gen,
        task: work.task,
        member: work.member.take(),
        fitness,
    };
    let url = format!("{}/simulations/{}/work_results?get_work=true", base_url, id);
    // Work around reqwest issue with "Connection: close", don't re-use client.
    let client = reqwest::Client::new();

    // upload work results and request more work
    let resp = client.post(&url)
      .json(&result)
      .send().await?
      .json::<GetWorkResp>().await?;

    Ok(resp.work)
}

async fn get_work(base_url: &str, id: Option<Uuid>) -> Result<Option<WorkUnit>, reqwest::Error> {
    let url = if let Some(sim_id) = id {
        format!("{}/simulations/{}/get_work", base_url, sim_id)
    } else {
        format!("{}/get_work", base_url)
    };

    // Work around reqwest issue with "Connection: close", don't re-use client.
    let client = reqwest::Client::new();
    let resp = client.get(&url)
      .send().await?
      .json::<GetWorkResp>().await?;

    Ok(resp.work)
}

fn do_cal_fitness(work: &mut WorkUnit, data: &TrainingSet) -> Option<f32> {
    if let Some(mut member) = work.member.take() {
        Some(data.solve(&mut member))
    } else {
        None
    }
}

fn do_training(work: &mut WorkUnit, data: &TrainingSet) {
    let train = work.train.as_ref()
        .unwrap_or(&TrainDto{ epochs: 100, learning_rate: 0.3});
    if let Some(member) = &mut work.member {
        data.train(&train, member);

        // show it
        data.show(member);
    }
}

async fn do_work(cache: &mut CacheSimData, base_url: &str, mut work: WorkUnit) -> Result<bool, reqwest::Error> {

    // get problem data for simulation.
    let sim_id = work.sim_id;
    println!("start working on simulation: {}", sim_id);
    let data = cache.get_sim_data(base_url, sim_id).await?;

    loop {
        let fitness = match work.task {
            SimTaskType::CalFitness => {
                do_cal_fitness(&mut work, &data)
            },
            SimTaskType::TrainBest => {
                do_training(&mut work, &data);
                None
            },
        };

        print!("*");
        flush();
        // upload work results and get more work.
        if let Some(new_work) = work_results(base_url, sim_id, &mut work, fitness).await? {
            work = new_work;
        } else {
            println!();
            break;
        }
    }
    Ok(true)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    env_logger::init();
    let base_url = "http://0.0.0.0:42069";

    let mut cache = CacheSimData::new();

    let mut sleep_time = 200;
    loop {
        let work = get_work(&base_url, None).await?;
        if let Some(work) = work {
            // reset sleep time
            sleep_time = 200;
            do_work(&mut cache, &base_url, work).await?;
        } else {
            println!("no work sleep");
            delay_for(Duration::from_millis(sleep_time)).await;
            // sleep longer if no work.
            if sleep_time < 2000 {
                sleep_time += 200;
            }
        }
    }
}

