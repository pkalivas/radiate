
extern crate radiate;

use std::error::Error;
use radiate::prelude::*;




fn main() -> Result<(), Box<dyn Error>> {

    let mut neat_env = NeatEnvironment::new()
        .set_input_size(2)
        .start_innov_counter();

    let mut net = Neat::new()
        .dense(7, &mut neat_env, Activation::Sigmoid)
        .dense(7, &mut neat_env, Activation::Sigmoid)
        .dense(1, &mut neat_env, Activation::Sigmoid);
        
    let xor = XOR::new();
    for _ in 0..5000 {
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


// unsafe impl Send for XOR {}
// unsafe impl Sync for XOR {}




// impl Problem<Neat> for XOR {

//     fn empty() -> Self { XOR::new() }

//     fn solve(&self, model: &Neat) -> f64 {
//         let mut total = 0.0;
//         for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
//             match model.feed_forward(&ins) {
//                 Ok(guess) => total += (guess[0] - outs[0]).powf(2.0),
//                 Err(_) => panic!("Error in training NEAT")
//             }
//         }
//         4.0 - total
//     }

// }