extern crate radiate;
extern crate radiate_web;
extern crate serde;
extern crate serde_derive;

use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};

use uuid::Uuid;

use radiate::prelude::*;
use radiate_web::prelude::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrainingSet {
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

    pub fn train(&self, train: &TrainDto, model: &mut Neat) {
        let num_train = train.epochs as usize;
        model.train(&self.inputs, &self.answers, train.learning_rate, Loss::Diff, |iter, _| {
            iter == num_train
        }).unwrap();
    }

    pub fn show(&self, model: &mut Neat) {
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SimTaskType {
    CalFitness,
    TrainBest,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkStatus {
    Queued,
    Running(Instant),
    Finished(Duration),
}

#[derive(Debug, Clone, Copy)]
pub struct SimTask {
    pub task: SimTaskType,
    pub status: WorkStatus,
    pub member_idx: Option<usize>,
}

impl SimTask {
    pub fn new(task: SimTaskType, member_idx: Option<usize>) -> Self {
        Self {
            task,
            status: WorkStatus::Queued,
            member_idx,
        }
    }

    pub fn reset(&mut self) {
        self.status = WorkStatus::Queued;
    }
}

impl Default for SimTask {
    fn default() -> Self {
        Self::new(SimTaskType::CalFitness, None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkUnit {
    pub id: usize,
    pub sim_id: Uuid,
    pub curr_gen: usize,
    pub task: SimTaskType,
    pub member_idx: Option<usize>,
    pub member: Option<Neat>,
    pub train: Option<TrainDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetWorkResp {
    pub work: Option<WorkUnit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetWorkResult {
    pub id: usize,
    pub curr_gen: usize,
    pub task: SimTaskType,
    pub member: Option<Neat>,
    pub fitness: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Evolving,
    Training,
    Finished,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationStatus {
    pub status: Status,
    pub curr_gen: usize,
    pub curr_epoch: usize,
    pub curr_fitness: Option<f32>,
    pub last_gen_elapsed: Option<Duration>,
    pub solution: Option<Neat>,
}

pub struct Simulation {
    id: Uuid,

    status: Status,
    population: Population<Neat, NeatEnvironment, TrainingSet>,

    // generation work queue
    work: Vec<SimTask>,
    work_queued: usize,
    work_running: usize,
    work_expired: usize,
    last_finished: Instant,
    work_expire_timeout: Duration,

    // generations (evolving)
    curr_gen: usize,
    curr_gen_start: Instant,
    last_gen_elapsed: Option<Duration>,
    num_gen: usize,

    // training
    curr_epoch: usize,
    solution: Option<Neat>,

    // current best fitness score
    curr_fitness: Option<f32>,
    target_fitness: Option<f32>,

    // Problem data.
    train: TrainDto,
    data: TrainingSet,
}

impl Simulation {
    /// build a simulation from a `RadiateDto`
    pub fn new_from(mut radiate: RadiateDto) -> Option<Self> {
        let pop = radiate.population?;
        let size = pop.size.unwrap_or(100);

        // prepare work queue
        let mut work = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            work.push(SimTask::new(SimTaskType::CalFitness, Some(i)));
        }

        // take out the training variables
        let train = radiate.train?;
        let num_evolve = pop.num_evolve.unwrap_or(50);

        // set up the population now that it has been recieved
        let mut population = Population::<Neat, NeatEnvironment, TrainingSet>::new()
            .size(size)
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
            status: Status::Evolving,
            population,

            work,
            work_queued: size as usize,
            work_running: 0,
            work_expired: 0,
            work_expire_timeout: Duration::from_secs(5),
            last_finished: Instant::now(),

            curr_gen: 0,
            curr_gen_start: Instant::now(),
            last_gen_elapsed: None,
            num_gen: num_evolve as usize,

            curr_epoch: 0,
            solution: None,

            curr_fitness: None,
            target_fitness: pop.target_fitness,

            train,
            data,
        })
    }

    /// simulation's uuid
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// get the simulation's training set.
    pub fn get_training_set(&self) -> &TrainingSet {
        &self.data
    }

    /// Get the simulations current status
    pub fn get_status(&self) -> SimulationStatus {
        let mut status = SimulationStatus {
          status: self.status,
          curr_gen: self.curr_gen,
          curr_epoch: self.curr_epoch,
          last_gen_elapsed: self.last_gen_elapsed,
          curr_fitness: self.curr_fitness,
          solution: None,
        };
        if self.status == Status::Finished {
            status.solution = self.solution.clone();
        }
        status
    }

    /// get a mutable refrence to a member in the current population
    pub fn member_mut(&mut self, idx: usize) -> Option<&mut Container<Neat, NeatEnvironment>> {
        self.population.member_mut(idx)
    }

    /// get an immutable refrence to a member in the current population
    pub fn member(&self, idx: usize) -> Option<&Container<Neat, NeatEnvironment>> {
        self.population.member(idx)
    }

    /// Get the simulation's solution
    pub fn get_solution(&self) -> Option<Neat> {
        self.solution.clone()
    }

    /// check for expired work units
    pub fn has_expired_work(&self) -> bool {
        self.work_expired > 0
    }

    /// check if this simulation has work
    pub fn has_work(&self) -> bool {
        if self.status == Status::Finished {
            // simulation finished.  no work.
            false
        } else if self.work_queued > 0 {
            // has queued work.
            true
        } else if self.has_expired_work() {
            // no queued work.  has expired work units.
            true
        } else {
            // no work available.
            false
        }
    }

    /// Prepare the work queue for the next generation.
    fn reset_work(&mut self) {
        for work in self.work.iter_mut() {
            work.reset();
        }
        self.work_queued = self.work.len();
        self.work_running = 0;
        self.last_finished = Instant::now();
    }

    /// check for end of generation.
    fn finished_work(&mut self) {
        // update number of running jobs.
        self.work_running -= 1;
        self.last_finished = Instant::now();
        // check if all queued & running jobs have finished.
        if self.work_running == 0 && self.work_queued == 0 {
            self.reset_work();
            match self.status {
                Status::Evolving => {
                    // handle end of generation calculations.
                    self.end_generation();
                },
                Status::Training => {
                    self.end_training();
                },
                _ => {
                },
            }
        }
    }

    /// handle the work results from a worker's WorkUnit.
    pub fn work_results(&mut self, result: GetWorkResult) {
        // make sure the results are for the current generation.
        if result.curr_gen != self.curr_gen {
            // results for old generation.
            return;
        }

        if let Some(work) = self.work.get_mut(result.id) {
            // check if the results is for the correct task type.
            if work.task != result.task {
                // old results from generation, ignore.
                return;
            }
            // check if work has already finished.
            match work.status {
                WorkStatus::Queued => {
                    // re-scheduled?
                },
                WorkStatus::Running(start) => {
                    work.status = WorkStatus::Finished(start.elapsed());
                    match work.task {
                        SimTaskType::CalFitness => {
                            // get member
                            let member = work.member_idx
                              .and_then(|idx| self.member_mut(idx));
                            if let Some(member) = member {
                                // update fitness
                                if let Some(fitness) = result.fitness {
                                    member.set_fitness(fitness);
                                }
                                // update member Genome
                                if let Some(new_member) = result.member {
                                    member.update_member(new_member);
                                }
                            }
                        },
                        SimTaskType::TrainBest => {
                            if let Some(new_member) = result.member {
                                self.solution = Some(new_member);
                            }
                        },
                    }
                    // check if generation has finished.
                    self.finished_work();
                },
                WorkStatus::Finished(_) => {
                    // work has already finished.  Ignore old results.
                },
            }
        }
    }

    /// Convert a simulation task into a WorkUnit.
    fn work_to_job(&mut self, id: usize, work: SimTask) -> Option<WorkUnit> {
        let mut job = WorkUnit {
            id,
            sim_id: self.id,
            curr_gen: self.curr_gen,
            task: work.task,
            member_idx: work.member_idx,
            member: None,
            train: None,
        };
        match job.task {
            SimTaskType::CalFitness => {
                if let Some(idx) = work.member_idx {
                    job.member = self.member(idx).map(|cont| cont.member.read().unwrap().clone());
                }
            },
            SimTaskType::TrainBest => {
                job.member = self.solution.clone();
                job.train = Some(self.train.clone());
            },
        }

        Some(job)
    }

    /// Get a work unit for this simulation if there is queued or expired work.
    pub fn get_work(&mut self) -> Option<WorkUnit> {
        if self.has_work() {
            let work = self.get_queued_work().or_else(|| {
                // get next expired work.
                self.get_expired_work()
            });
            if let Some((id, work)) = work {
                return self.work_to_job(id, work);
            }
        } else {
            // check for expired work.
            if self.work_running > self.work_expired {
                self.find_expired_work();
            }
        }
        None
    }

    /// Find queued work.
    fn get_queued_work(&mut self) -> Option<(usize, SimTask)> {
        // TODO: track index of next task to avoid looping.
        for (id, work) in self.work.iter_mut().enumerate() {
            if work.status == WorkStatus::Queued {
                work.status = WorkStatus::Running(Instant::now());
                self.work_queued -= 1;
                self.work_running += 1;
                return Some((id, *work));
            }
        }
        return None;
    }

    /// Update count of expired work.
    fn find_expired_work(&mut self) {
        let mut expired = 0;
        for work in self.work.iter_mut() {
            match work.status {
                WorkStatus::Running(start) => {
                    if start.elapsed() > self.work_expire_timeout {
                        expired += 1;
                    }
                },
                _ => {
                    // ignore finished and queued work.
                },
            }
        }
        self.work_expired = expired;
    }

    /// Find an expired job.
    fn get_expired_work(&mut self) -> Option<(usize, SimTask)> {
        for (id, work) in self.work.iter_mut().enumerate() {
            match work.status {
                WorkStatus::Running(start) => {
                    if start.elapsed() > self.work_expire_timeout {
                        self.work_expired -= 1;
                        work.status = WorkStatus::Running(Instant::now());
                        return Some((id, *work));
                    }
                },
                _ => {
                    // ignore finished and queued work.
                },
            }
        }
        return None;
    }

    /// Handle end of generation.
    fn end_generation(&mut self) {
        if let Some((fit , top)) = self.population.end_generation() {
            println!("epoch: {} score: {}", self.curr_gen, fit);

            // update generation stats.
            self.curr_fitness = Some(fit);
            self.last_gen_elapsed = Some(self.curr_gen_start.elapsed());
            self.curr_gen_start = Instant::now();

            // check if we have reached the last generation.
            let finished = if self.curr_gen == self.num_gen {
                true
            } else if let Some(target_fitness) = self.target_fitness {
                // check if we have reached the target fitness
                if fit > target_fitness {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if finished {
                self.solution = Some(top);
                self.start_training();
            } else {
                self.curr_gen += 1;
            }
        } else {
            unreachable!("End generation failed.  This shouldn't be possible.");
        }
    }

    /// Start training of best member.
    fn start_training(&mut self) {
        self.status = Status::Training;
        self.work.clear();
        self.work.push(SimTask::new(SimTaskType::TrainBest, None));
        self.work_queued = 1;
    }

    /// When training has finished the simulation is finished.
    fn end_training(&mut self) {
        self.status = Status::Finished;
        // TODO: allow workers to update training epoch
        self.curr_epoch = self.train.epochs as usize;
    }
}
