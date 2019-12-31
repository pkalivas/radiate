
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
        .lstm(4, 1);

    
    let num_evolve = 500;
    let (mut solution, _) = Population::<Neat, NeatEnvironment, MemoryTest>::new()
        .constrain(neat_env)
        .size(100)
        .populate_clone(starting_net)
        .debug(true)
        .dynamic_distance(true)
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.5,
            species_target: 5
        })
        .stagnation(15, vec![
            Genocide::KillWorst(0.9)
        ])
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            let diff = 1.0 - fit;
            num == num_evolve || (diff > 0.0 && diff < 0.01)
        })?;
        
        println!("\nTime in millis: {}", thread_time.elapsed().as_millis());
        MemoryTest::new().show(&mut solution);

        // let mut test_net = Neat::new()
        //     .input_size(1)
        //     .lstm(6, 1);
        //     // .dense_pool(1, Activation::Sigmoid);

        
        // let m = MemoryTest::new();

        // println!("\n\n\n");
        // for _ in 0..500 {
        //     m.backprop(&mut test_net);
        // }
        // m.show(&mut test_net);


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
            let guess = model.feed_forward(&i).unwrap();
            println!("Input: {:?}, Output: {:?}, Guess: {:.2}", i, o, guess[0]);
        }
    }

    pub fn backprop(&self, model: &mut Neat) {
        for (i, o) in self.input.iter().zip(self.output.iter()) {
            model.backprop(i, o, 0.3, true);
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
            match model.feed_forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        total /= self.input.len() as f32;
        1.0 - total
    }
}

