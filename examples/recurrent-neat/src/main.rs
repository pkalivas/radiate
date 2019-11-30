
extern crate radiate;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;




fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let mut neat_env = NeatEnvironment::new()
        .set_input_size(1)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.6)
        .set_new_node_rate(0.02)
        .set_new_edge_rate(0.03)
        .set_reactivate(0.2)
        .set_c1(1.0)
        .set_c2(1.0)
        .set_c3(0.003)
        .set_node_types(vec![NodeType::Recurrent])
        .set_activation_functions(vec![Activation::Tahn])
        .start_innov_counter();


    let mut starting_net = Neat::base(&mut neat_env);
    let num_backprop = 500;
    let num_evolve = 100;
    let ism = ISM::new();
    let mut epochs = 0;
    
    let solution = loop {
        let (mut solution, env) = Population::<Neat, NeatEnvironment, ISM>::new()
            .constrain(neat_env)
            .size(150)
            .populate_clone(starting_net)
            .debug(true)
            .dynamic_distance(true)
            .configure(Config {
                inbreed_rate: 0.001,
                crossover_rate: 0.50,
                distance: 4.0,
                species_target: 8
            })
            .stagnation(15, vec![
                Genocide::KeepTop(10)
            ])
            .run(|_, fit, num| {
                println!("Generation: {} score: {}", num, fit);
                num == num_evolve
            })?;

            for _ in 0..num_backprop {
                ism.backprop(&mut solution);
            }
            if epochs == 10 {
                break solution;
            }
            starting_net = solution;
            neat_env = env;
            epochs += 1;
        };            

    let ism = ISM::new();
    println!("{:#?}", ism);
    let total = ism.solve(&solution);


    println!("Solution: {:#?}", solution);
    solution.see();
    println!("Time in millis: {}", thread_time.elapsed().as_millis());
    ism.show(&solution);
    println!("Total: {}", total);

    Ok(())
}






#[derive(Debug)]
pub struct ISM {
    inputs: Vec<Vec<f64>>,
    answers: Vec<Vec<f64>>
}



impl ISM {
    pub fn new() -> Self {
        ISM {
            inputs: vec![
                vec![56.0],
                vec![57.6],
                vec![56.6],
                vec![55.3],
                vec![55.5],
                vec![56.7],
                vec![56.5],
                vec![59.3],
                vec![60.2],
                vec![58.5],
                vec![58.2],
                vec![59.3],
                vec![60.2],
                vec![58.5],
                vec![58.2],
                vec![59.3],
                vec![59.1],
                vec![60.7],
                vec![59.3],
                vec![57.9],
                vec![58.7],
                vec![60.0],
                vec![58.4],
                vec![60.8],
                vec![59.5],
                vec![57.5],
                vec![58.8],
                vec![54.3],
                vec![56.6],
                vec![54.2],
                vec![55.3],
                vec![52.8],
                vec![52.1],
                vec![51.7],
                vec![51.2],
                vec![49.1],
                vec![47.8],
            ],
            answers: vec![
                vec![57.6],
                vec![56.6],
                vec![55.3],
                vec![55.5],
                vec![56.7],
                vec![56.5],
                vec![59.3],
                vec![60.2],
                vec![58.5],
                vec![58.2],
                vec![59.3],
                vec![60.2],
                vec![58.5],
                vec![58.2],
                vec![59.3],
                vec![59.1],
                vec![60.7],
                vec![59.3],
                vec![57.9],
                vec![58.7],
                vec![60.0],
                vec![58.4],
                vec![60.8],
                vec![59.5],
                vec![57.5],
                vec![58.8],
                vec![54.3],
                vec![56.6],
                vec![54.2],
                vec![55.3],
                vec![52.8],
                vec![52.1],
                vec![51.7],
                vec![51.2],
                vec![49.1],
                vec![47.8],
                vec![48.3]
            ]
        }.normalize()
    }


    fn normalize(self) -> Self {
        let inputs_min = ISM::minimum(&self.inputs);
        let inputs_max = ISM::maximum(&self.inputs);
        let output_min = ISM::minimum(&self.answers);
        let output_max = ISM::maximum(&self.answers);

        ISM {
            inputs: self.inputs.iter()
                .map(|x| {
                    vec![(x[0] - inputs_min) / (inputs_max - inputs_min)]
                })
                .collect(),
            answers: self.answers.iter()
                .map(|x| {
                    vec![(x[0] - output_min) / (output_max - output_min)]
                })
                .collect()
        }
    }

    fn minimum(nums: &Vec<Vec<f64>>) -> f64 {
        nums.iter()
            .fold(1000.0, |min, curr| {
                if curr[0] < min {
                    return curr[0]
                }
                min
            })
    }

    fn maximum(nums: &Vec<Vec<f64>>) -> f64 {
        nums.iter()
            .fold(-1000.0, |max, curr| {
                if curr[0] > max {
                    return curr[0]
                }
                max
            })
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


unsafe impl Send for ISM {}
unsafe impl Sync for ISM {}




impl Problem<Neat> for ISM {

    fn empty() -> Self { ISM::new() }

    fn solve(&self, model: &Neat) -> f64 {
        let mut total = 0.0;
        let mut goal = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.feed_forward(&ins) {
                Ok(guess) => total += (guess[0] - outs[0]).powf(2.0),
                Err(_) => panic!("Error in training NEAT")
            }
            goal += outs[0];
        }
        goal - total
    }

}