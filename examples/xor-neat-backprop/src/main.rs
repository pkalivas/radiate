


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
    for _ in 0..100 {
        xor.backprop(&mut net);
    }

    xor.show(&mut net);
    println!("Time in millis: {}", thread_time.elapsed().as_millis());

    let mut lstm = Neat::new()
        .input_size(2)
        .lstm(25, 2);

    let output = lstm.feed_forward(&vec![1.0, 1.0]);
    println!("otuput: {:?}", output);

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



