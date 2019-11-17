
## Radiate

Radiate is a parallel genetic programming engine capable of evolving solutions for supervised, unsupervised, and general reinforcement learning problems. 

Radiate follows the evolutionary algorithm found in [NEAT](http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) (Neuroevolution of Augmented Topologies), but seperates the the evolutionary process from the graph datastructure allowing users to evolve any defined datastructure. NEAT describes an evolutionary algorithm which follows speciation through fitness sharing thus allowing species to optimize within their own niche. 

Radiate exposes three traits to the user which (dpending on the problem) must be implemented:
1. **Genome**  
Genome wraps the structure to be evolved and makes the user implement two nessesary functions and one optional. Distance and crossover must be implemented but base is optional (depending on how the user chooses to fill the population).
2. **Environment**  
Environment represnts the evolutionary enviromnet for the genome, this means it can contain simple statistics for the population's evoltuion, or parameters for crossing over and distance. Internally it is wrapped in a mutable threadsafe pointer so it is not intended to be shared for each genome, rather one per population. Environment requires no implementations of it's one function, however depending on the use case envionment exposes a function called reset which is intended to 'reset' the envionment.
3. **Problem**  
Problem is what gives a genome it's fitness score. It requires two implemented functions: empty and solve. Empty is required and should return a base problem (think new()). Solve takes a genome and returns that genome's fitness score, so this is where the analyzing of the current state of the genome occurs. Checkout the examples folder for a few examples of how this can be implemented.

Radiate also comes with two models already built. Those being Evtree, and NEAT. Both come with default environments, however due to the amount of impact small changes can have on evolution, users might want to use different settings.

**Evtree**  
is a twist on decision trees where instead of using a certain split criteria like the gini index, each node in the tree has a collection of matrices and uses these matrices to decide which subtree to explore. This algorithm is something I created and although I'm sure it's been built before, I haven't found any papers or implementations of anything like it. It is a binary tree and is only good for classification right now. I currently have plans to make it a little more verbose through better matrix mutliplication, propagating inputs further through the tree, and possibly introducing multiple sub trees and regression - however these increase the compute time.   
**NEAT**  
is the algorithm described by Kenneth O. Stanley in the paper linked above. I've tried to follow the rules in the paper pretty well and have implemented some things I've found online as well such as historical marking control, and dynamic distance for speciation. The dynamic distance between species is available for any structure, however the speciation through historical markings described in the paper is only good for NEAT. Neat exposes a few different activation functions for one to choose from, but mutliple can be used at once and each new node will choose one randonly. This NEAT implementation also includes a backpropagation function which operates much like traditional neural networks which propagate the input error back through the network and adjust the weights. This alone is useless, however in pair with the evolution engine, can produce very nice and quick results. 

The population is pretty easy to set up assuming the all traits have been implemented. Here is a quick glance of setting up a problem and evolution engine to evolve the weights on a NEAT graph to equal 100. 
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
    
    println!("Solution");
    println!("{:#?}", solution);
    
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

_examples of Evtree and NEAT can be found in ./examples using the xor problem to optimize the structures_ 
