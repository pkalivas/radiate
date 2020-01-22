
## Radiate
[![Build Status](https://travis-ci.com/pkalivas/radiate.svg?branch=master)](https://travis-ci.com/pkalivas/radiate)
![Crates.io](https://img.shields.io/crates/v/radiate)

## Versions
**1.1.2** - For forward and backward passes of NEAT, gated propagation in LSTM layers is now run in parallel which cuts training times in half. Changed the readme to be a full implementation of the engine which is a little more helpful for setting everything up. Added another readme file to radiate/src/models/ which gives examples of setting up a NEAT neural network.

**1.1.1** - Fixed dumb bug in NEAT which was causing a error in backprop.

**1.0.9** - **As of 1/10/2020 all versions after 1.0.9 require the nightly toolchain** Added serialization and deserialization to NEAT model through serde integration - serializing trait objects requires nightly crates for now.

Coming from Evolutionary Radiation.
> Evolutionary radiation is a rapid increase in the number of species with a common ancestor, characterized by great ecological and morphological diversity - Pascal Neige.

Radiate is a parallel genetic programming engine capable of evolving solutions to many problems as well as training learning algorithms.

Radiate seperates the the evolutionary process from the object it is evolving, allowing users to evolve any defined structure. The algorithm follows an evolutionary process through speciation thus allowing the structures to optimize within their own niche. 

Radiate exposes three traits to the user which must be implemented:
1. **Genome**  
Genome wraps the structure to be evolved and makes the user implement two nessesary functions and one optional. Distance and crossover must be implemented but base is optional (depending on how the user chooses to fill the population).
2. **Environment**  
Environment represnts the evolutionary enviromnet for the genome, this means it can contain simple statistics for the population's evoltuion, or parameters for crossing over and distance. Internally it is wrapped in a mutable threadsafe pointer so it is not intended to be shared for each genome, rather one per population. Environment requires no implementations of it's one function, however depending on the use case envionment exposes a function called reset which is intended to 'reset' the envionment.
3. **Problem**  
Problem is what gives a genome it's fitness score. It requires two implemented functions: empty and solve. Empty is required and should return a base problem (think new()). Solve takes a genome and returns that genome's fitness score, so this is where the analyzing of the current state of the genome occurs.

Radiate also comes with two models already built. Those being Evtree, and NEAT.

### Evtree
Is a twist on decision trees where instead of using a certain split criteria like the gini index, each node in the tree has a collection of matrices and uses these matrices to decide which subtree to explore. This algorithm is something I created and although I'm sure it's been built before, I haven't found any papers or implementations of anything like it. It is a binary tree and is only good for classification right now. 
### NEAT
Also known as Neuroevolution of Augmented Topologies, is the algorithm described by Kenneth O. Stanley in [this](http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) paper. This NEAT implementation also includes a backpropagation function which operates much like traditional neural networks which propagate the input error back through the network to adjust the weights. In pair with the evolution engine, can produce very nice and quick results. NEAT lets the use define how the network will be constructed, whether that be in a traitional neural network fashion where layers are stacked next to each other or with evolutionary topolgies of the graph through as explained in the paper. This means NEAT can be used in an evolutionary sense, through forward propagation and back propagation, or any combination of the two. There are examples of both in /examples.
 **more color on Neat in radiate/src/models/**

## Setup
The population is pretty easy to set up assuming the all traits have been implemented. The population is a higher abstraction to keep track of varibales used during evoltuion but not needed within epoch - things like the problem, solution, to print to the screen, ect. A new population is filled originally with default settings:
```rust
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
        environment: Arc::new(Mutex::new(E::default())),
        // determine which genomes will live on and passdown to the next generation
        survivor_criteria: SurvivalCriteria::Fittest,
        // determine how to pick parents to reproduce
        parental_criteria: ParentalCriteria::BiasedRandom
    }
}
```
## Example
A quick and easy example of implementing all the needed traits and running the genetic engine to generate a string print "hello world!". There are examples of how to run a Neat neural network in radiate/src/models/. 
To run this:
```bash
git clone https://github.com/pkalivas/radiate.git
cd radiate
cargo build --verbose && cargo run --bin helloworld
```
On my computer (Windows 10, x64-based, i7-7700 @ 4.20GHz, 32GB RAM) this finishes in less than half a second.
```rust
extern crate radiate;
extern crate rand;

use std::error::Error;
use std::time::Instant;
use std::sync::{Arc, RwLock};
use rand::Rng;
use radiate::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let thread_time = Instant::now();
    let (top, _) = Population::<Hello, HelloEnv, World>::new()
        .size(100)
        .populate_base()
        .dynamic_distance(true)
        .stagnation(10, vec![Genocide::KillWorst(0.9)])
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.5,
            species_target: 5
        })
        .run(|model, fit, num| {
            println!("Generation: {} score: {:.3?}\t{:?}", num, fit, model.as_string());
            fit == 12.0 || num == 500
        })?;
        
    println!("\nTime in millis: {}, solution: {:?}", thread_time.elapsed().as_millis(), top.as_string());
    Ok(())
}
```
Now create the problem which holds the target and actually scores the solvers. 
Note the target data isn't being coppied for each solver.
```rust
pub struct World { target: Vec<char> }

impl World {
    pub fn new() -> Self {
        World {
            target: vec!['h', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!']
        }
    }
}

impl Problem<Hello> for World {

    fn empty() -> Self { World::new() }

    fn solve(&self, model: &mut Hello) -> f32 {
        let mut total = 0.0;
        for (index, letter) in self.target.iter().enumerate() {
            if letter == &model.data[index] {
                total += 1.0;
            }
        }
        total        
    }
}
```
Now define an environment to hold global data for crossover and distance, things like record/stat keeping, crossover probabilities, really anything that is needed globally is held in this.
```rust
#[derive(Debug, Clone)]
pub struct HelloEnv {
    pub alph: Vec<char>,
}

impl HelloEnv {
    pub fn new() -> Self {
        HelloEnv {
            alph: vec!['!', ' ', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'], // now i know my abcs..
        }
    }
}

/// implement Environment and default for the HelloEnv, Environment is there in case you want the environment to be dynamic
impl Envionment for HelloEnv {}
impl Default for HelloEnv {
    fn default() -> Self {
        Self::new()
    }
}
```
Finally, define a solver. This is type 'Genome' in the evolutionary process and makes up the population of each generation.
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Hello {
    pub data: Vec<char>
}

impl Hello {
    pub fn new(alph: &Vec<char>) -> Self {
        let mut r = rand::thread_rng();
        Hello { 
            data: (0..12)
                .map(|_| alph[r.gen_range(0, alph.len())])
                .collect() 
        }
    }

    pub fn as_string(&self) -> String {
        self.data
            .iter()
            .map(|x| String::from(x.to_string()))
            .collect::<Vec<_>>()
            .join("")
    }
}

/// implement genome for Hello
impl Genome<Hello, HelloEnv> for Hello {

    fn crossover(parent_one: &Hello, parent_two: &Hello, env: &Arc<RwLock<HelloEnv>>, crossover_rate: f32) -> Option<Hello> {
        let mut r = rand::thread_rng();
        let params = env.read().unwrap();
        let mut new_data = Vec::new();
        
        if r.gen::<f32>() < crossover_rate {
            for (one, two) in parent_one.data.iter().zip(parent_two.data.iter()) {
                if one != two {
                    new_data.push(*one);
                } else {
                    new_data.push(*two);
                }
            }
        } else {
            new_data = parent_one.data.clone();
            let swap_index = r.gen_range(0, new_data.len());
            new_data[swap_index] = params.alph[r.gen_range(0, params.alph.len())];
        }
        Some(Hello { data: new_data })
    }

    fn distance(one: &Hello, two: &Hello, _: &Arc<RwLock<HelloEnv>>) -> f32 {
        let mut total = 0_f32;
        for (i, j) in one.data.iter().zip(two.data.iter()) {
            if i == j {
                total += 1_f32;
            }
        }
        one.data.len() as f32 / total
    }

    fn base(env: &mut HelloEnv) -> Hello {
        Hello::new(&env.alph)
    }
}
```
Running this looks something like this when running in the cmd:
```bash
Generation: 100 score: 8.000    "!eulozworlde"
Generation: 101 score: 8.000    "!eulozworlde"
Generation: 102 score: 8.000    "!eulozworlde"
Generation: 103 score: 8.000    "!eulozworlde"
Generation: 104 score: 8.000    "!eulozworlde"
Generation: 105 score: 9.000    "heulozworlde"
Generation: 106 score: 9.000    "heulozworlde"
Generation: 107 score: 9.000    "heulozworlde"
Generation: 108 score: 9.000    "heulozworlde"
Generation: 109 score: 9.000    "heulozworlde"
Generation: 110 score: 9.000    "heulozworlde"
Generation: 111 score: 9.000    "heulozworlde"
Generation: 112 score: 10.000   "heulo worlde"
Generation: 113 score: 10.000   "heulo worlde"
Generation: 114 score: 10.000   "heulo worlde"
Generation: 115 score: 10.000   "heulo worlde"
Generation: 116 score: 10.000   "heulo worlde"
Generation: 117 score: 10.000   "heulo worlde"
Generation: 118 score: 10.000   "heulo worlde"
Generation: 119 score: 10.000   "heulo worlde"
Generation: 120 score: 11.000   "hello worlde"
Generation: 121 score: 11.000   "hello worlde"
Generation: 122 score: 11.000   "hello worlde"
Generation: 123 score: 11.000   "hello worlde"
Generation: 124 score: 11.000   "hello worlde"
Generation: 125 score: 11.000   "hello worlde"
Generation: 126 score: 12.000   "hello world!"

Time in millis: 349, solution: "hello world!"
```
This comes right now with four examples, just run "cargo run --bin (desired example name)" to run any of them
1. **xor-evtree**
2. **xor-neat**
3. **xor-neat-backprop**
4. **lstm-neat**
5. **helloworld**

## Create a Population
The initial generation in the population can be created in four different ways depending on the user's use case. The examples show different ways of using them.
1. **populate_gen** - Give the population an already constructed Generation struct. 
2. **populate_base** - Create a generation of Genomes from the Genome's base function.
3. **populate_vec** - Take a vec and populate the generation from the Genomes in the vec.
4. **populate_clone** - Given a single Genome, clone it size times and create a generation from the clones.

## Speciation
Because the engine is meant to evolve Genomes through speciation, the Config struct is meant to hold parameters for the speciation of the population, adjusting these will change the way the Genomes are split up within the population and thus drive the discovery of new Genomes through crossover and mutation.

## Genocide
During evolution it can be common for either the population or specific species to become stagnat or stuck at a certain point in the problem space. To mend this, population allows the user to define a number of stagnant generations until a 'genocide' will occur. These genocide options can be found in genocide.rs and are simply ways to clean the population to give the Genome's an opportunity to breath and evolve down a new path in the problem space. 
```rust
pub enum Genocide {
    KeepTop(usize),
    KillWorst(f32),
    KillRandom(f32),
    KillOldestSpecies(usize)
}
```
This is definitly an area which can be improved in the algorithm.

