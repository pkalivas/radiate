extern crate rayon;

use std::sync::{Arc, Mutex};
use std::marker::Sync;
use std::fmt::Debug;
use std::cmp::PartialEq;
use rayon::prelude::*;
use super::{
    generation::{Generation, Container},
    genome::Genome,
    problem::{Problem},
    environment::{Envionment},
    genocide::{Genocide},
};




struct Stagnant {
    target_stagnation: usize,
    current_stagnation: usize,
    previous_top_score: f64,
    cleaners: Vec<Genocide>
}


/// This is just to keep track of a few parameters for 
/// the population, this encapsulates a few arguments for speication 
/// in the algorithm, these are specific to genetic algorithms 
/// which implement speciation between members of the population.
/// It also leaves room for more paramters to be added in the future.
#[derive(Clone)]
pub struct Config {
    pub inbreed_rate: f32,
    pub crossover_rate: f32,
    pub distance: f64,
    pub species_target: usize
}


/// Population is what facilitates the evolution from a 5000 ft view
/// keeping track of what the generation is doing, marking statistics
/// down from each one, and holding resource sensitive things like
/// datasets as well as making sure that the optimization is moving 
/// forward through the stats it keeps (stagnation)
pub struct Population<T, E, P>
    where
        T: Genome<T, E> + Send + Sync,
        E: Envionment + Sized + Send + Sync,
        P: Problem<T>
{
    size: i32,
    dynamic_distance: bool,
    debug_progress: bool,
    config: Config,
    curr_gen: Generation<T, E>,
    stagnation: Stagnant,
    solve: Arc<P>,
    environment: Arc<Mutex<E>>
}




/// implmenet the population
impl<T, E, P> Population<T, E, P>
    where
        T: Genome<T, E> + Send + Sync,
        E: Envionment + Sized + Send + Sync + Default,
        P: Problem<T>
{

    /// This creates a base popuation with a generation which is randomly generated 
    /// this serves as a jumping point for the rest of the evolution
    pub fn new() -> Self {   
        Population {
            // define the number of members to participate in evolution and be injected into the current generation
            size: 100,
            // determin if the species should be aiming for a specific number of species by adjusting the distance threshold
            dynamic_distance: false,
            // debug_progress is only used to print out some information from each generation
            // to the console during training to get a glimps into what is going on
            debug_progress: false,
            // create a new config to help the speciation of the population
            config: Config::new(),
            // create a new empty generation to be passed down through the population 
            curr_gen: Generation::<T, E>::new(),
            // keep track of fitness score stagnation through the population
            stagnation: Stagnant::new(0, Vec::new()),
            // Arc<Problem> so the problem can be sent between threads safely without duplicating the problem, 
            // if the problem gets duplicated every time a supervised learning problem with a lot of data could take up a ton of memory
            solve: Arc::new(P::empty()),
            // create a new solver settings that will hold the specific settings for the defined solver 
            // that will allow the structure to evolve through generations
            environment: Arc::new(Mutex::new(E::default()))
        }
    }



    /// Each generation will be trained by a call to this function 
    /// resulting optimization of the current generation, up to a 
    /// crossover into the next generation which will be set to the 
    /// new current generation
    #[inline]
    pub fn train(&mut self) -> Option<(f64, T)>
        where 
            T: Genome<T, E> + Clone + Send + Sync + Debug + PartialEq,
            P: Send + Sync
    {
        // optimize the population and return the top member 
        let top_member = self.curr_gen.optimize(self.solve.clone())?;
        // adjust the distance of the population if needed
        if self.dynamic_distance { self.adjust_distance(); }
        // speciate the generation into niches then see if the population is stagnant
        // if the population is stagnant, clean the population 
        self.curr_gen.speciate(self.config.distance, &self.environment);
        self.manage_stagnation(top_member.0);
        // If debug is set to true, this is the place to show it before the new generation is 
        if self.debug_progress { self.show_progress(); }
        // create a new generation and return it
        self.curr_gen = self.curr_gen.create_next_generation(self.size, self.config.clone(), &self.environment)?;
        // return the top member score and the member
        Some((top_member.0, (*top_member.1).clone()))
    }

    

    /// Check to see if the population is stagnant or not, if it is 
    /// then go ahead and clean the population 
    fn manage_stagnation(&mut self, curr_top_score: f64) {
        if self.stagnation.target_stagnation == self.stagnation.current_stagnation {
            for cleaner in self.stagnation.cleaners.iter() {
                cleaner.kill(&mut self.curr_gen);
            }
            self.stagnation.current_stagnation = 0;
        } else if curr_top_score == self.stagnation.previous_top_score {
            self.stagnation.current_stagnation += 1;
        } else {
            self.stagnation.current_stagnation = 0;
        }
        self.stagnation.previous_top_score = curr_top_score;
    }



    /// dynamically adjust the distance of a popualtion 
    fn adjust_distance(&mut self) {
        if self.curr_gen.species.len() < self.config.species_target {
            self.config.distance -= 0.5;
        } else if self.curr_gen.species.len() > self.config.species_target {
            self.config.distance += 0.5;
        }
        if self.config.distance < 0.3 {
            self.config.distance = 0.3;
        }
    }



    /// Run the population according to a user defined function, the inputs of which
    /// are a borrowed member which is the top member of the current generation, 
    /// the fitness of that member, and the current number of generations.
    /// This function will continue until this function returns a true value 
    pub fn run<F>(&mut self, runner: F) -> Result<(T, E), &'static str>
        where 
            F: Fn(&T, f64, i32) -> bool + Sized,
            T: Genome<T, E> + Clone + Send + Sync + Debug + PartialEq,
            P: Send + Sync,
            E: Clone
    {
        let mut index = 0;
        loop {
            match self.train() {
                Some(result) => {
                    let (fit, top) = result;
                    if runner(&top, fit, index) {
                        return Ok((top.clone(), (*self.environment.lock().unwrap()).clone()));
                    }
                    index += 1;
                },
                None => return Err("Error Training")
            }
        }
    }


    
    /// if debug is set to true, this is what will print out 
    /// the training to the screen during optimization.
    fn show_progress(&self) {
        println!("\n");
        for i in self.curr_gen.species.iter() {
            i.lock().unwrap().display_info();
        }
    }
    

    
    /////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// configure all the settings for the population these all have default settings if they are not set ///
    /// customly, however you might find those default settings do not satisfy the needs of your problem  ///
    /////////////////////////////////////////////////////////////////////////////////////////////////////////
    
    /// Set the beginning generation of the population by a generion object
    /// this can be done in three ways all listed below.
    /// 
    /// 1.) populate_gen - Create a generation object outsize of this scope and give it to the 
    ///                    population, return the popuolation back to the caller
    /// 2.) populate_bas - as long as the popualtion size has already been set and the type T has 
    ///                    implemented the base trait fn, this will generate a new base generation
    /// 3.) populate_vec - Give the population a vec of type T and generate a new generation from it 
    ///                    then return the population back to the caller
    /// 4.) populate_clone - Take a base type T and create a population that is made up 
    ///                      completely of clones of this type - they will all be the same 
    ///                      at least for the first generation, this is useful for algorithms like NEAT
    
    /// give the populate a direct generation object 
    pub fn populate_gen(mut self, gen: Generation<T, E>) -> Self {
        self.curr_gen = gen;
        self
    }
    
    /// populate the populate with the base implementation of the genome 
    pub fn populate_base(mut self) -> Self 
    where P: Send + Sync
    {
        self.curr_gen = Generation {
            members: (0..self.size)
            .into_par_iter()
            .map(|_| {
                let mut lock_set = self.environment.lock().unwrap();
                Container {
                    member: Arc::new(T::base(&mut lock_set)),
                    fitness_score: 0.0,
                    species: None
                }    
            })
            .collect(),
            species: Vec::new(),
        };
        self
    }
    
    /// given a vec of type T which implements Genome, populate the population
    pub fn populate_vec(mut self, vals: Vec<T>) -> Self {
        self.curr_gen = Generation {
            members: vals.into_iter()
            .map(|x| {
                Container {
                    member: Arc::new(x),
                    fitness_score: 0.0,
                    species: None
                }
            })
            .collect(),
            species: Vec::new()
        };
        self
    }
    
    /// Given one type T which is a genome, create a population with clones of the original
    pub fn populate_clone(mut self, original: T) -> Self 
    where T: Genome<T, E> + Clone 
    {
        self.curr_gen = Generation {
            members: (0..self.size as usize)
            .into_iter()
            .map(|_| {
                Container {
                    member: Arc::new(original.clone()),
                    fitness_score: 0.0,
                    species: None
                }
            })
            .collect(),
            species: Vec::new()
        };
        self
    }
    
    /// Give solver settings to the population to evolve the strucutre defined 
    pub fn constrain(mut self, environment: E) -> Self {
        self.environment = Arc::new(Mutex::new(environment));
        self
    }
    
    /// Set the size of the population, the population size
    /// will default to 100 if this isn't set which could be enough 
    /// depending on the problem being solved 
    pub fn size(mut self, size: i32) -> Self {
        self.size = size;
        self
    }
    
    /// set the dynamic distance bool
    pub fn dynamic_distance(mut self, opt: bool) -> Self {
        self.dynamic_distance = opt;
        self
    }
    
    /// set the stagnation number of the population
    pub fn stagnation(mut self, stag: usize, cleaner: Vec<Genocide>) -> Self {
        self.stagnation = Stagnant::new(stag, cleaner);
        self
    }
   
    /// Set a config object to the population, these are arguments related
    /// to evolution through speciation, so these are all speciation
    /// arguments
    pub fn configure(mut self, spec: Config) -> Self {
        self.config = spec;
        self
    }
    
    /// Impose a problem on the population, in other words, 
    /// give the population a problem to solve. This 
    /// will default to an empty problem, meaning the population
    /// will not solve anything if this isn't set. This is really
    /// the most important arguemnt for the population
    pub fn impose(mut self, prob: P) -> Self {
        self.solve = Arc::new(prob);
        self
    }
    
    /// debug determines what to display to the screen during evolution
    pub fn debug(mut self, d: bool) -> Self {
        self.debug_progress = d;
        self
    }


}




/// This is a default config implementation which 
/// needs to be set for the population to evolve 
/// with speciation. These numbers need to be 
/// set for the evolution to work correctly
impl Config {
    pub fn new() -> Self {
        Config {
            inbreed_rate: 0.0,
            crossover_rate: 0.0,
            distance: 0.0,
            species_target: 0
        }
    }
}




impl Stagnant {
    pub fn new(target_stagnation: usize, cleaners: Vec<Genocide>) -> Self {
        Stagnant {
            target_stagnation,
            current_stagnation: 0,
            previous_top_score: 0.0,
            cleaners
        }
    }
}
