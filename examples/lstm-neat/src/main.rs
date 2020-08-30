
extern crate radiate;
extern crate serde_json;
extern crate uuid;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;


fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let neat_env = NeatEnvironment::new()
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.5)
        .set_new_node_rate(0.14)
        .set_new_edge_rate(0.14)
        .set_recurrent_neuron_rate(1.0)
        .set_reactivate(0.2)
        .set_activation_functions(vec![
            Activation::Sigmoid,
            Activation::Relu,
        ]);
        
    let num_evolve = 500;

    let data = MemoryTest::new();
    let starting_net = Neat::new()
        .input_size(1)
        .batch_size(data.output.len())
        // .gru(10, 5, Activation::Tanh)
        .dense_pool(1, Activation::Sigmoid);
        // .lstm(10, 1, Activation::Sigmoid);

    let (mut solution, _) = Population::<Neat, NeatEnvironment, MemoryTest>::new()
        .constrain(neat_env)
        .size(200)
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
        
    //let num_train = 0;
        // solution.train(&data.input, &data.output, 0.01, Loss::Diff, |iter, loss| {
        //     let temp = format!("{:.6}", loss).parse::<f32>().unwrap().abs();
        //     println!("epoch: {:?} loss: {:.6?}", iter, temp);
        //     iter == num_train || (temp < 1_f32 && temp % 1.0 == 0.0)
        // })?;

        
        solution.reset();
        data.show(&mut solution);
        solution.reset();

        println!("Score: {:?}\nTime in millis: {}", data.solve(&mut solution), thread_time.elapsed().as_millis());
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
            let guess = model.forward(&i).unwrap();
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
            match model.forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        total /= self.input.len() as f32;
        1.0 - total
    }
}

