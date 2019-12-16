


extern crate radiate;

use std::error::Error;
use radiate::prelude::*;


fn main() -> Result<(), Box<dyn Error>> {

    let mut neat_env = NeatEnvironment::new()
        .set_input_size(2)
        .start_innov_counter();

    let mut net = Neat::new()
        .dense(7, &mut neat_env, Activation::Relu)
        .dense(7, &mut neat_env, Activation::Relu)
        .dense(1, &mut neat_env, Activation::Sigmoid);
        
    let xor = XOR::new();
    for _ in 0..500 {
        xor.backprop(&mut net);
    }
    xor.show(&mut net);


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

    fn backprop(&self, model: &mut Neat) {
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            model.backprop(i, o, 0.3);
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



