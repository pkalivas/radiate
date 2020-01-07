
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
Also known as Neuroevolution of Augmented Topologies, is the algorithm described by Kenneth O. Stanley in [this](http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) paper. This NEAT implementation also includes a backpropagation function which operates much like traditional neural networks which propagate the input error back through the network and adjust the weights. In pair with the evolution engine, can produce very nice and quick results. Neat exposese a few different options to the user through enums. These enums are mapped to easily extensable logic under the hood. NEAT lets the use define how the network will be constructed, whether that be in a traitional neural network fashion where layers are stacked next to each other or with evolutionary topolgies of the graph through as explained in the paper. This means NEAT can be used in an evolutionary sense, through forward propagation and back propagation, or any combination of the two. There are examples of both in /examples.
Currently there are two available layers with more on the way.
```rust
pub enum LayerType {
    Dense,      // typical dense layer of a neural network with no ability to evolve its strucutre 
    DensePool,  // the algorithm described in the paper meaning a fully functional neural network can be evolved through one dense pool layer
    LSTM,       // uses dense pool for evoution and traditional backpropagation through time for training.
}
```
All neural networks need nonlinear functions to represent complex datasets. Neat 
allows users to specify which activation function a neuron will use through a customizable vec! in the neat enviornment. If more than one is specified, it will be randomly chosen when a neuron is created.
```rust
pub enum Activation {
    Sigmoid,       // default
    Tahn,
    Relu,
    Softmax,
    LeakyRelu(f32),
    ExpRelu(f32),
    Linear(f32)   
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
        F: Fn(&T, f32, i32) -> bool + Sized,
        T: Genome<T, E> + Clone + Send + Sync + PartialEq,
        P: Send + Sync,
        E: Clone
```
## Example
Quick example of evolving an lstm network for a few generations then fine-tuning it with backpropagation. Uncommenting out freestyle() will let the trained model predict off of it's previous output and continue on as so.
To run this:
```bash
git clone https://github.com/pkalivas/radiate.git
cd radiate
cargo build --verbose && cargo run --bin xor-neat-backprop
```
On my computer (Windows 10, x64-based, i7-7700 @ 4.20GHz, 32GB RAM) this finishes in about 6 seconds.
```rust
extern crate radiate;
use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let neat_env = NeatEnvironment::new()
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.7)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.2)
        .set_activation_functions(vec![
            Activation::Sigmoid,
            Activation::Relu,
        ]);

    let starting_net = Neat::new()
        .input_size(1)
        .lstm(1, 1);        
    
    let num_evolve = 100;
    let (mut solution, _) = Population::<Neat, NeatEnvironment, MemoryTest>::new()
        .constrain(neat_env)
        .size(100)
        .populate_clone(starting_net)
        .debug(true)
        .dynamic_distance(true)
        .stagnation(15, vec![Genocide::KillWorst(0.9)])
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.5,
            species_target: 5
        })
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            num == num_evolve
        })?;
        
        let data = MemoryTest::new();
        MemoryTest::new().show(&mut solution);
        solution.train(&data.input, &data.output, 200, 0.3, 7)?;
        println!("{:#?}", solution);

        // data.freestyle(12, &mut solution);
        data.show(&mut solution);
    
        solution.reset();
        println!("Score: {:?}\n\nTime in millis: {}", data.solve(&mut solution), thread_time.elapsed().as_millis());    
        Ok(())
}
 
#[derive(Debug)]
pub struct MemoryTest {
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>
}

impl MemoryTest {
    pub fn new() -> Self {
        MemoryTest {
            input: vec![vec![0.0], vec![0.0], vec![0.0], vec![1.0], vec![0.0], vec![0.0], vec![0.0]],
            output: vec![vec![0.0], vec![0.0], vec![1.0], vec![0.0], vec![0.0], vec![0.0], vec![1.0]]
        }
    }

    pub fn show(&self, model: &mut Neat) {
        for (i, o) in self.input.iter().zip(self.output.iter()) {
            let guess = model.forward(&i).unwrap();
            println!("Input: {:?}, Output: {:?}, Guess: {:.2}", i, o, guess[0]);
        }
        println!("\nTest next few inputs:");
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![1.0], vec![0.0], model.forward(&vec![1.0]).unwrap()[0]);
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![0.0], vec![0.0], model.forward(&vec![0.0]).unwrap()[0]);
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![0.0], vec![0.0], model.forward(&vec![0.0]).unwrap()[0]);
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![0.0], vec![1.0], model.forward(&vec![0.0]).unwrap()[0]);
    }

    pub fn freestyle(&self, iters: usize, model: &mut Neat) {
        let round = |x| {
            if x < 0.5 { 0.0 } else { 1.0 }
        };  
        let expec = vec![vec![0.0], vec![0.0], vec![0.0], vec![1.0]];
        let mut guess = round(model.forward(&vec![1.0]).unwrap()[0]);
        let mut counter: usize = 1;
        println!("\nFreestyling with rounding\n\nInput: {:?}, Expecting: {:?}, Guess: {:.2}", vec![1.0], vec![0.0], guess);
        for _ in 0..iters {
            let temp = guess;
            guess = round(model.forward(&vec![temp]).unwrap()[0]);
            println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", temp, expec[counter % 4], guess);
            counter += 1;
        }
    }
}

unsafe impl Send for MemoryTest {}
unsafe impl Sync for MemoryTest {}

impl Problem<Neat> for MemoryTest {

    fn empty() -> Self { MemoryTest::new() }
    
    fn solve(&self, model: &mut Neat) -> f32 {
        let mut total = 0.0;
        for (ins, outs) in self.input.iter().zip(self.output.iter()) {
            match model.forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        total /= self.input.len() as f32;
        1.0 - total
    }
}
```
Neat can also be constructed as a traditional neural network as such:
```rust
extern crate radiate;
use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
       
    let thread_time = Instant::now();
    let mut net = Neat::new()
        .input_size(2)
        .dense(7, Activation::Relu)
        .dense(7, Activation::Relu)
        .dense(1, Activation::Sigmoid);
        
    let xor = XOR::new();
    net.train(&xor.inputs, &xor.answers, 100, 0.3, 1);
    xor.show(&mut net);

    println!("Time in millis: {}", thread_time.elapsed().as_millis());
    Ok(())
}

#[derive(Debug)]
pub struct XOR {
    inputs: Vec<Vec<f32>>,
    answers: Vec<Vec<f32>>
}

impl XOR {
    pub fn new() -> Self {
        XOR {
            inputs: vec![
                vec![0.0, 0.0],
                vec![1.0, 1.0],
                vec![1.0, 0.0],
                vec![0.0, 1.0],
            ],
            answers: vec![
                vec![0.0],
                vec![0.0],
                vec![1.0],
                vec![1.0],
            ]
        }
    }

    fn show(&self, model: &mut Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = model.feed_forward(&i).unwrap();
            println!("Guess: {:.2?} Answer: {:.2}", guess, o[0]);
        }
    }
}
```
This comes right now with four examples, just run "cargo run --bin (desired example name)" to run any of them
1. **xor-evtree**
2. **xor-neat**
3. **xor-neat-backprop**
4. **lstm-neat**

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
This is definitly an area which can be improved in the algorithm.



_examples of Evtree and NEAT can be found in ./examples using the xor problem to optimize the structures_ 
 
