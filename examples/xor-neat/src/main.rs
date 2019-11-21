
extern crate radiate;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;




fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let mut neat_env = NeatEnvironment::new()
        .set_input_size(3)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.6)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.2)
        .set_c1(1.0)
        .set_c2(1.0)
        .set_c3(0.03)
        .set_activation_functions(vec![
            Activation::Sigmoid,
            Activation::Relu,
        ])
        .start_innov_counter();

    let starting_net = Neat::base(&mut neat_env);
    let (mut solution, _) = Population::<Neat, NeatEnvironment, XOR>::new()
        .constrain(neat_env)
        .size(150)
        .populate_clone(starting_net)
        .debug(false)
        .dynamic_distance(true)
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.50,
            distance: 3.0,
            species_target: 10
        })
        .stagnation(15, vec![
            Genocide::KeepTop(5)
        ])
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            let diff = 4.0 - fit;
            (diff > 0.0 && diff < 0.01) || num == 500
        })?;
        

    let xor = XOR::new();
    let total = xor.solve(&solution);

    println!("Solution: {:#?}", solution);
    solution.see();
    println!("Time in millis: {}", thread_time.elapsed().as_millis());
    xor.show(&solution);
    println!("Total: {}", total);

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
                vec![0.0, 0.0, 1.5],
                vec![1.0, 1.0, 1.5],
                vec![1.0, 0.0, 1.5],
                vec![0.0, 1.0, 1.5],
            ],
            answers: vec![
                vec![0.0],
                vec![0.0],
                vec![1.0],
                vec![1.0],
            ]
        }
    }


    fn show(&self, model: &Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = model.feed_forward(&i).unwrap();
            println!("Guess: {:.2?} Answer: {:.2}", guess, o[0]);
        }
    }

    fn backprop(&self, model: &mut Neat) {
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            model.backprop(i, o, 0.1);
        }
    }

}


unsafe impl Send for XOR {}
unsafe impl Sync for XOR {}




impl Problem<Neat> for XOR {

    fn empty() -> Self { XOR::new() }

    fn solve(&self, model: &Neat) -> f64 {
        let mut total = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.feed_forward(&ins) {
                Ok(guess) => total += (guess[0] - outs[0]).powf(2.0),
                Err(_) => panic!("Error in training NEAT")
            }
        }
        4.0 - total
    }

}