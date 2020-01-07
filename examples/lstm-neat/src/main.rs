
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
        .set_new_node_rate(0.05)
        .set_new_edge_rate(0.05)
        .set_reactivate(0.2)
        .set_activation_functions(vec![
            Activation::Sigmoid,
            Activation::Relu,
        ]);


    let starting_net = Neat::new()
        .input_size(1)
        .lstm(1, 1);

    let num_evolve = 75;
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
        println!("Score: {:?}", data.solve(&mut solution));
        println!("\nTime in millis: {}", thread_time.elapsed().as_millis());
        
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



    pub fn freestyle(&self, iters: usize, model: &mut Neat) {
        println!("Freestyling");
        let round = |x| {
            if x < 0.5 {
                0.0
            } else {
                1.0
            }
        };  
        
        let expec = vec![vec![0.0], vec![0.0], vec![0.0], vec![1.0]];
        let mut guess = round(model.forward(&vec![1.0]).unwrap()[0]);
        let mut counter: usize = 1;
        println!("Input: {:?}, Expecting: {:?}, Guess: {:.2}", vec![1.0], vec![0.0], guess);
        
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

