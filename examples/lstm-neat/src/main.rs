
extern crate radiate;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;



fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let mut neat_env = NeatEnvironment::new()
        .set_input_size(2)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.75)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.2)
        .set_c1(1.0)
        .set_c2(1.0)
        .set_c3(0.04)
        .set_activation_functions(vec![
            Activation::Sigmoid,
            Activation::Relu,
        ])
        .start_innov_counter();
        
    let starting_net = Neat::new()
        .input_size(1)
        .lstm(25, 1);
        
    let num_evolve = 5000;
    let xor = XOR::new();


    let (mut solution, _) = Population::<Neat, NeatEnvironment, XOR>::new()
        .constrain(neat_env)
        .size(200)
        .populate_clone(starting_net)
        .debug(false)
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
            let diff = 4.0 - fit;
            (diff > 0.0 && diff < 0.01) || num == num_evolve
        })?;
        

    println!("{:#?}", solution);
    xor.show(&mut solution);
    println!("total: {}", xor.solve(&mut solution));
    println!("Time in millis: {}", thread_time.elapsed().as_millis());
    Ok(())
}




#[derive(Debug)]
pub struct XOR {
    inputs: Vec<Vec<f64>>,
    answers: Vec<Vec<f64>>
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


unsafe impl Send for XOR {}
unsafe impl Sync for XOR {}




impl Problem<Neat> for XOR {

    fn empty() -> Self { XOR::new() }

    fn solve(&self, model: &mut Neat) -> f64 {
        let mut total = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.feed_forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        4.0 - total
    }

}