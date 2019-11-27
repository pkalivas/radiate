
## Radiate
[![Build Status](https://travis-ci.com/pkalivas/radiate.svg?branch=master)](https://travis-ci.com/pkalivas/radiate)
![Crates.io](https://img.shields.io/crates/v/radiate)

Coming from Evolutionary Radiation.
> Evolutionary radiation is a rapid increase in the number of species with a common ancestor, characterized by great ecological and morphological diversity - Pascal Neige.

Radiate is a parallel genetic programming engine capable of evolving solutions for supervised, unsupervised, and general reinforcement learning problems. 

Radiate seperates the the evolutionary process from the object it is evolving, allowing users to evolve any defined structure. The algorithm follows an evolutionary process through speciation thus allowing the structures to optimize within their own niche. 

Radiate exposes three traits to the user which (depending on the problem) must be implemented:
1. **Genome**  
Genome wraps the structure to be evolved and makes the user implement two nessesary functions and one optional. Distance and crossover must be implemented but base is optional (depending on how the user chooses to fill the population).
2. **Environment**  
Environment represnts the evolutionary enviromnet for the genome, this means it can contain simple statistics for the population's evoltuion, or parameters for crossing over and distance. Internally it is wrapped in a mutable threadsafe pointer so it is not intended to be shared for each genome, rather one per population. Environment requires no implementations of it's one function, however depending on the use case envionment exposes a function called reset which is intended to 'reset' the envionment.
3. **Problem**  
Problem is what gives a genome it's fitness score. It requires two implemented functions: empty and solve. Empty is required and should return a base problem (think new()). Solve takes a genome and returns that genome's fitness score, so this is where the analyzing of the current state of the genome occurs. Checkout the examples folder for a few examples of how this can be implemented.

Radiate also comes with two models already built. Those being Evtree, and NEAT. Both come with default environments, however due to the amount of impact small changes can have on evolution, users might want to use different settings.

### Evtree
Is a twist on decision trees where instead of using a certain split criteria like the gini index, each node in the tree has a collection of matrices and uses these matrices to decide which subtree to explore. This algorithm is something I created and although I'm sure it's been built before, I haven't found any papers or implementations of anything like it. It is a binary tree and is only good for classification right now. I currently have plans to make it a little more verbose through better matrix mutliplication, propagating inputs further through the tree, and possibly introducing multiple sub trees and regression - however these increase the compute time.   
### NEAT
Also known as Neuroevolution of Augmented Topologies, is the algorithm described by Kenneth O. Stanley in [this](http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) paper. I've tried to follow the rules in the paper pretty well and have implemented some things I've found online as well such as historical marking control, and dynamic distance for speciation. The dynamic distance between species is available for any structure, however the speciation through historical markings described in the paper is only good for NEAT. Neat exposes a few different activation functions for one to choose from, but mutliple can be used at once and each new node will choose one randonly. This NEAT implementation also includes a backpropagation function which operates much like traditional neural networks which propagate the input error back through the network and adjust the weights. This alone is useless, however in pair with the evolution engine, can produce very nice and quick results. Neat exposese a few different options to the user through enums. These enums are mapped to easily extensable logic under the hood. Through the neat enviornment, users can customize which node type or combination of any of them, will be used to create the neat graph. *Note these enums do not need to be explicitly defined and will default to Dense nodetype with a Sigmoid activation function.*
```rust
pub enum NodeType {
    Dense,      // implemented and the default
    LSTM,       // coming soon
    Recurrent   // coming soon
}
```
All neural networks need nonlinear functions to represent complex datasets. Neat 
allows users to specify which activation function a neuron will use through a customizable vec! in the neat enviornment. If more than one is specified, it will be randomly chosen when a neuron is created.
```rust
pub enum Activation {
    Sigmoid,       // default
    Tahn,
    Relu,
    LeakyRelu(f64),
    ExpRelu(f64),
    Linear(f64)   
}
```

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
The run() function must be the last function chained to the population because it takes a closure which when returns true, returns back the top Genome as it's initial type and the environment. The closure given to run receives a borrowed type T which is a Genome, that genome's fitness, and the current epoch.
```rust
pub fn run<F>(&mut self, runner: F) -> Result<(T, E), &'static str>
    where 
        F: Fn(&T, f64, i32) -> bool + Sized,
        T: Genome<T, E> + Clone + Send + Sync + PartialEq,
        P: Send + Sync,
        E: Clone
```
## Example
Quick example of optimizing the NEAT algorithm to find a graph where the sum of all edges is .0001 away from 100.
To run this:
```bash
git clone https://github.com/pkalivas/radiate.git
cd radiate
cargo build --verbose && cargo run --bin wmax-neat
```
On my computer (Windows 10, x64-based, i7-7700 @ 4.20GHz, 32GB RAM) this finishes in about 15 seconds.

```rust
extern crate radiate;

use std::error::Error;
use radiate::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {

    let mut neat_env = default_neat_env();
    let starting_net = Neat::base(&mut neat_env);
    let (solution, environment) = Population::<Neat, NeatEnvironment, NeatWeightMax>::new()
        .constrain(neat_env)
        .size(250)
        .populate_clone(starting_net)
        .debug(true)
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 3.0,
            species_target: 4
        })
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            (100.0 - fit).abs() > 0.0 && (100.0 - fit).abs() < 0.0001
        })?;
    
    println!("Solution\n{:#?}", solution);
    
    Ok(())
}

struct NeatWeightMax;

impl Problem<Neat> for NeatWeightMax {
    fn empty() -> Self { NeatWeightMax }

    fn solve(&self, model: &Neat) -> f64 {
        let mut total = 0.0;
        for edge in model.edges.values() {
            total += edge.weight;
        }
        100.0 - (100.0 - total).abs()
    }
}
```
This comes right now with four examples, just run "cargo run --bin (desired example name)" to run any of them
1. **wmax-neat**
2. **xor-evtree**
3. **xor-neat**
4. **xor-neat-backprop**

I'm going to add more examples soon, thinking about doing a Knapsack and nqueens example.

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
I find that keep top is probably the most useful, this is definitly an area which can be improved in the algorithm.



_examples of Evtree and NEAT can be found in ./examples using the xor problem to optimize the structures_ 
 
