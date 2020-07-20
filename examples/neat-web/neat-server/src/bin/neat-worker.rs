#![feature(proc_macro_hygiene, decl_macro)]

extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate reqwest;

use std::time::Duration;
use std::io::{self, Write};

use std::sync::Arc;
use std::collections::{HashMap, hash_map::Entry};

use uuid::Uuid;

use env_logger;
use anyhow::Result;

use tokio::task;
use tokio::sync::RwLock;
use tokio::time::delay_for;
 
use radiate_web::*;
use radiate::prelude::*;

use neat_server::*;

fn flush() {
  io::stdout().flush().ok().expect("Failed to flush stdout")
}

#[derive(Debug, Default, Clone)]
struct CacheSimData {
    training_data: Arc<RwLock<HashMap<Uuid, Arc<TrainingSet>>>>,
}

impl CacheSimData {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn get_sim_data(&self, base_url: &str, id: Uuid) -> Result<Arc<TrainingSet>> {
        let mut training_data = self.training_data.write().await;
        let entry = training_data.entry(id);
        // check cache.
        match entry {
            Entry::Occupied(val) => {
                // got cached value.
                let data = val.get();
                Ok(data.clone())
            },
            Entry::Vacant(val) => {
                let url = format!("{}/simulations/{}/training_set", base_url, id);
                // Work around reqwest issue with "Connection: close", don't re-use client.
                let client = reqwest::Client::new();
                let data = Arc::new(client.get(&url)
                  .send().await?
                  .json::<TrainingSet>().await?);

                val.insert(data.clone());
                Ok(data)
            },
        }
    }
}

async fn work_results(base_url: &str, id: Uuid, mut work: WorkUnit, fitness: Option<f32>) -> Result<Option<WorkUnit>> {
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

async fn get_work(base_url: &str, id: Option<Uuid>) -> Result<Option<WorkUnit>> {
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

async fn do_work_unit(mut work: WorkUnit, data: Arc<TrainingSet>) -> Result<(WorkUnit, Option<f32>)> {
    Ok(task::spawn_blocking(move || {
        let fitness = match work.task {
            SimTaskType::CalFitness => {
                do_cal_fitness(&mut work, &data)
            },
            SimTaskType::TrainBest => {
                do_training(&mut work, &data);
                None
            },
        };
        (work, fitness)
    }).await?)
}

async fn do_work(cache: &CacheSimData, base_url: &str, work: WorkUnit) -> Result<usize> {
    // get problem data for simulation.
    let sim_id = work.sim_id;
    let data = cache.get_sim_data(base_url, sim_id).await?;

    let mut work_count = 1;

    let mut next_work = work;
    loop {
        let (work, fitness) = do_work_unit(next_work, data.clone()).await?;

        print!("*");
        flush();
        // upload work results and get more work.
        if let Some(new_work) = work_results(base_url, sim_id, work, fitness).await? {
            next_work = new_work;
            work_count += 1;
        } else {
            break;
        }
    }
    Ok(work_count)
}

async fn worker(id: usize, cache: CacheSimData, base_url: String) -> Result<()> {
    let mut sleep_time = 2000;
    loop {
        let work = get_work(&base_url, None).await?;
        if let Some(work) = work {
            // reset sleep time
            sleep_time = 200;
            let count = do_work(&cache, &base_url, work).await?;
            println!("worker({}) count = {}", id, count);
        } else {
            delay_for(Duration::from_millis(sleep_time)).await;
            // sleep longer if no work.
            if sleep_time < 2000 {
                sleep_time += 200;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let num_workers = std::env::args().nth(1).unwrap_or("4".to_string())
        .parse::<usize>().expect("Expected number of workers to spawn.");

    let base_url = "http://0.0.0.0:42069";

    let cache = CacheSimData::new();

    let mut workers = Vec::with_capacity(num_workers);
    for id in 0..num_workers {
          workers.push(task::spawn(worker(id, cache.clone(), base_url.to_string())));
    }
    // block until workers finish.
    for worker in workers.drain(..) {
        worker.await.unwrap()?;
    }
    Ok(())
}
