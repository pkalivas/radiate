
extern crate radiate;
extern crate radiate_matrix_tree;
extern crate simple_matrix;

use std::error::Error;
use std::time::Instant;
use simple_matrix::Matrix;
use radiate::prelude::*;
use radiate_matrix_tree::prelude::{Evtree, TreeEnvionment, default_evtree_env};



fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let tree_env = default_evtree_env();
    Population::<Evtree, TreeEnvionment, XOR>::new()
        .impose(XOR::new())
        .constrain(tree_env)
        .size(500)
        .populate_base()
        .debug(true)
        .stagnation(10, vec![
            Genocide::KillOldestSpecies(3)
        ])
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.15,
            species_target: 4
        })
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            fit == 4.0
        })?;
    
    println!("TIME: {}", thread_time.elapsed().as_millis());

    Ok(())
}




#[derive(Debug)]
pub struct XOR {
    inputs: Vec<Vec<f32>>,
    answers: Vec<Vec<f32>>
}



impl XOR {
    pub fn new() -> Self {
        XOR {
            inputs: vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![0.0, 1.0],
                vec![1.0, 1.0]
            ],
            answers: vec![
                vec![0.0],
                vec![1.0],
                vec![1.0],
                vec![0.0]
            ]
        }
    }
}


impl Problem<Evtree> for XOR {

    fn empty() -> Self { XOR::new() }

    fn solve(&self, model: &mut Evtree) -> f32 {
        let mut total = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            let temp_cpy: Vec<f32> = (0..ins.len()).map(|x| ins[x]).collect();
            let curr_input = Matrix::from_iter(ins.len(), 1, temp_cpy);
            let model_output = (model).propagate(curr_input);
            if model_output as f32 == outs[0] {
                total += 1.0;
            }
        }
        total
    }

}
