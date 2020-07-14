#![feature(proc_macro_hygiene, decl_macro)]

extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;
extern crate reqwest;

use std::time::Duration;

use uuid::Uuid;

use env_logger;

use tokio::time::delay_for;
use reqwest::Client;
 
use radiate::prelude::*;
use neat_server::*;

async fn sim_get_data(client: &Client, base_url: &str, id: Uuid) -> Result<TrainingSet, reqwest::Error> {
    let url = format!("{}/simulations/{}/training_set", base_url, id);
    let training_set = client.get(&url)
        .send().await?
        .json::<TrainingSet>().await?;
    //println!("training_set = {:?}", training_set);

    Ok(training_set)
}

async fn update_work(client: &mut Client, base_url: &str, id: Uuid, work: &WorkJob, fitness: Option<f32>, member: Option<Neat>) -> Result<SimulationStatus, reqwest::Error> {
    let result = WorkResult {
        id: work.id,
        curr_gen: work.curr_gen,
        task: work.task,
        member,
        fitness,
    };
    let url = format!("{}/simulations/{}/update_work", base_url, id);
    let mut cnt = 0usize;
    loop {
        let res = match client.post(&url).json(&result).send().await {
            Ok(resp) => {
                resp.json::<SimulationStatus>().await
            },
            Err(err) => {
                Err(err)
            }
        };
        match res {
            Ok(status) => {
                return Ok(status);
            },
            Err(err) => {
                println!("update_work() failed: {:?}", err);
                cnt += 1;
                if cnt > 5 {
                    return Err(err);
                }
                //client = reqwest::Client::new();
            }
        }
        //println!("status = {:?}", status);
    }
}

async fn get_work(client: &mut Client, base_url: &str, id: Option<Uuid>) -> Result<Option<(WorkJob, Neat)>, reqwest::Error> {
    let url = if let Some(sim_id) = id {
        format!("{}/simulations/{}/get_work", base_url, sim_id)
    } else {
        format!("{}/get_work", base_url)
    };

    let mut cnt = 0usize;
    loop {
        let res = match client.get(&url).send().await {
            Ok(resp) => {
                resp.json::<DoWork>().await
            },
            Err(err) => {
                Err(err)
            }
        };
        match res {
            Ok(work) => {
                //println!("work = {:?}", work.work);
                return Ok(work.work);
            },
            Err(err) => {
                println!("get_work() failed: {:?}", err);
                cnt += 1;
                if cnt > 5 {
                    return Err(err);
                }
            }
        }
        //println!("status = {:?}", status);
    }
}

async fn do_work(client: &mut Client, base_url: &str, work: (WorkJob, Neat)) -> Result<bool, reqwest::Error> {
    // unwrap DoWork
    let (mut work, mut member) = work;

    // get problem data for simulation.
    let sim_id = work.sim_id;
    let data = sim_get_data(client, base_url, sim_id).await?;

    loop {
        let mut fitness = 0.0;
        match work.task {
            WorkTask::CalFitness => {
                fitness = data.solve(&mut member);
            },
            WorkTask::TrainBest => {
                // TODO:
            },
        }

        // upload work results.
        update_work(client, base_url, sim_id, &work, Some(fitness), Some(member)).await?;

        // try getting more work for same simulation.
        if let Some(new_work) = get_work(client, &base_url, Some(sim_id)).await? {
            work = new_work.0;
            member = new_work.1;
        } else {
            break;
        }
    }
    Ok(true)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    env_logger::init();
    let mut client = reqwest::Client::new();

    let base_url = "http://0.0.0.0:42069";

    loop {
        let work = get_work(&mut client, &base_url, None).await?;
        if let Some(work) = work {
            do_work(&mut client, &base_url, work).await?;
        } else {
            delay_for(Duration::from_millis(500)).await;
        }
    }
}

