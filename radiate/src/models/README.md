
### Working with NEAT
**Defining a neat instance**

*Each Neat::new() defaults to no layers, batch size of one, and input size of 0 - basically it won't work without chaining layers to it*

Define a new Neat network with an input size of one, batch size of 1, and a dense pool layer 
```rust
    let neural_network = Neat::new()
        .input_size(1)
        .dense_pool(1, Activation::Sigmoid);    // (output size, output layer activation)
```
Define a new Neat network with an input of 3, batch of 5, and an lstm layer
```rust
    let neural_network = Neat::new()
        .input_size(3)
        .batch_size(5)
        .lstm(10, 1, Activation::Sigmoid);      // (memory size, output size, output layer activation)
```
Define a new Neat network with multiple layers 
```rust
// note - this network will panic! if traditional backpropagation is used 
//        this is because backprop is not yet implemented for the gated 
//        recurrent unit layer (gru) yet. It can be evolved though.
    let neural_network = Neat::new()
        .input_size(1)
        .dense(5, Activation::Relu)
        .dense(5, Activation::Tanh)
        .gru(10, 10, Activation::Tanh)          // (memory size, output size, output layer activation)
        .dense_pool(1, Activation::Sigmoid);
```
NEAT currently has four layer types that can be chained onto a neat instance. 

**1.)** Dense - a normal dense layer of a neural network - not capable of evolving hidden neurons or other connections. Evolution instead only changes weights.

**2.)** DensePool - the algorithm described in the paper, capable of evolving hidden neurons and connections between them as well as changing weights.

**3.)** LSTM - a traditional long short term memory layer where each gate is a dense_pool network. Note - if the network is being evolved, the forward pass through an lstm is run syncronously, but if the network is being trained the forward pass and backward pass gated calculations are run in parallel which in my experience cuts training time in half.

**4.)** GRU - a simple gated recurrent unit layer. Like the lstm, this is made of multiple dense_pool networks which act as the gates. As of radiate v1.1.5 the gru is only viable for evoltuion and not for backpropagation, but implementing the backprop for gru is next on the todo list. This layer runs on a single thread.

All neural networks need nonlinear functions to represent complex datasets. Neat allows users to specify which activation function a neuron will use through a customizable vec! in the neat enviornment.
```rust
pub enum Activation {
    Sigmoid,
    Tanh,
    Relu,
    Softmax,       // Cannot be used on hidden neurons
    LeakyRelu(f32),
    ExpRelu(f32),
    Linear(f32)   
}
```

Examples on how to set up and run a NEAT network to evolve then train.

```rust
extern crate radiate;
extern crate serde_json;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {

    // set up a timer just to see how long it takes. Then define a NeatEnvironment to give
    // parameters to how the NEAT algorithm will be evolved. This controls how weights are edited,
    // how neurons are added, how connections are added, and which activation functions to use in hidden neurons
    let thread_time = Instant::now();
    let neat_env = NeatEnvironment::new()
        .set_weight_mutate_rate(0.8)        // 80% chance that the weights will be mutated, 20% change the weights will not be changed at all
        .set_edit_weights(0.1)              // 10% change that a weight will be assigned a new random number, 90% change it will be mutated by +/- weight_perturb
        .set_weight_perturb(1.7)            // if a weight is selected to be mutated, multiply the original weight by +/- 1.7 (shouldn't be larger than 2.0)
        .set_new_node_rate(0.4)             // if the layer is LSTM or dense_pool, 40% chance a new hidden neuron will be added
        .set_recurrent_neuron_rate(1.0)     // *v1.1.52* for every new neuron added, the % chance is recurrent 1.0 meaning 100%, 0.0 meaning 0% (not compatible with backprop)
        .set_new_edge_rate(0.4)             // if the layer is LSTM or dense_pool, 40% chance a new connection will be added between two random neurons with a random weight
        .set_reactivate(0.2)                // if the layer is LSTM or dense_pool, 20% chance a deactivated connection will be reactivated
        .set_activation_functions(vec![     // when new neurons are added, a random activation function is chosen from this list to give to the neuron
            Activation::Sigmoid,
            Activation::Relu,
        ]);
        
    // the number of generations to evolve then the number of epochs to train
    let num_evolve = 20;
    let num_train = 1000;

    // create a new problem and a new NEAT solver 
    let data = MemoryTest::new();
    let starting_net = Neat::new()
        .input_size(1)                      // set the input size
        .batch_size(data.output.len())      // set the number of forward passes before weights are updated
        .lstm(10, 1, Activation::Sigmoid);  // give the network one LSTM layer with a memory size of 10 and output size of 1 with a output activation of sigmoid

    // define and run a population to evolve the starting net
    let (mut solution, _) = Population::<Neat, NeatEnvironment, MemoryTest>::new()
        .constrain(neat_env)                                // give the population an environment to evolve in (evolutionary parameters defined above)
        .size(100)                                          // population size of 100
        .populate_clone(starting_net)                       // how to create the initial population (in this case, clone the starting_net 100 times)
        .debug(true)                                        // will print the species and their adjusted fitness scores at the end of each generation
        .dynamic_distance(true)                             // move the distance between networks to match the species_target specified below
        .stagnation(15, vec![Genocide::KillWorst(0.9)])     // if the fitness score of the best member doesn't improve in 15 genertaions, kill the worst 90% of the population
        .configure(Config {                                 //////////////// Configure the breeding parameters //////////////// 
            inbreed_rate: 0.001,                            // 0.1% chance to breed two members of the same species 
            crossover_rate: 0.75,                           // 75% chance two parents will be crossed over to create a child, 25% chance the most fit parent will be copied and mutated
            distance: 0.5,                                  // initial distance between species, if dynamic distance is true, this will chance to fit the species target
            species_target: 5                               // how many species you want, if dynaic distance is true, distance will move until this is met, if it is false, this might not work
        })
        .run(|_, fit, num| {                                // given the best member of the population, their fitness score, and the iteration number, return a bool. 
            println!("Generation: {} score: {}", num, fit); //      if this returns true, evolution will stop and return the top member from the population and the enviornment 
            num == num_evolve                               //      in this case, if the num equals the num_evolve, finish evoltuion
        })?;
        
        // NEAT allows for traditional training of neural networks as well given the (input data, target data, learning rate, loss function, and a function)
        // if the function (epoch number, epoch loss) returns true, finish training, otherwise continue
        solution.train(&data.input, &data.output, 0.3, Loss::Diff, |iter, loss| {
            let temp = format!("{:.4}", loss).parse::<f32>().unwrap().abs();
            println!("epoch: {:?} loss: {:.4?}", iter, temp);
            iter == num_train || (temp < 1_f32 && temp % 1.0 == 0.0)
        })?;

        // reset the NEAT network then show it 
        solution.reset();
        data.show(&mut solution);

        // reset the save and load the model
        solution.save("network.json")?;
        let mut net = Neat::load("network.json")?;

        // show the score on the data with the time it took to solve the problem
        println!("Score: {:?}\nTime in millis: {}", data.solve(&mut net), thread_time.elapsed().as_millis());
        Ok(())
}
 


/// This is a dummy test to make sure the lstm layer can remember things throughout time
/// the way it works is given 3 0's, the next output should be a 1
#[derive(Debug)]
pub struct MemoryTest {
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>
}

impl MemoryTest {
    pub fn new() -> Self {
        MemoryTest {
            input: vec![
                vec![0.0],
                vec![0.0],
                vec![0.0],
                vec![1.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
            ],
            output: vec![
                vec![0.0],
                vec![0.0],
                vec![1.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
                vec![1.0],
            ]
        }
    }


    pub fn show(&self, model: &mut Neat) {
        for (i, o) in self.input.iter().zip(self.output.iter()) {
            let guess = model.forward(i).unwrap();
            println!("Input: {:?}, Output: {:?}, Guess: {:.2}", i, o, guess[0]);
        }
        println!("\nTest next few inputs:");
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![1.0], vec![0.0], model.forward(&vec![1.0]).unwrap()[0]);
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![0.0], vec![0.0], model.forward(&vec![0.0]).unwrap()[0]);
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![0.0], vec![0.0], model.forward(&vec![0.0]).unwrap()[0]);
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![0.0], vec![1.0], model.forward(&vec![0.0]).unwrap()[0]);
    }
}


impl Problem<Neat> for MemoryTest {

    fn empty() -> Self { MemoryTest::new() }
    
    fn solve(&self, model: &mut Neat) -> f32 {
        let mut total = 0.0;
        for (ins, outs) in self.input.iter().zip(self.output.iter()) {
            match model.forward(ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        model.reset();
        total /= self.input.len() as f32;
        1.0 - total
    }
}
```
