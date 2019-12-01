
extern crate radiate;
extern crate csv;

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
        .set_weight_perturb(1.8)
        .set_new_node_rate(0.04)
        .set_new_edge_rate(0.03)
        .set_reactivate(0.2)
        .set_c1(1.0)
        .set_c2(1.0)
        .set_c3(0.003)
        .set_node_types(vec![
            NodeType::Recurrent,
        ])
        .set_activation_functions(vec![
            Activation::Tahn,
        ])
        .start_innov_counter();


    let starting_net = Neat::base(&mut neat_env);
    let (solution, _) = Population::<Neat, NeatEnvironment, ISM>::new()
        .constrain(neat_env)
        .size(150)
        .populate_clone(starting_net)
        .debug(true)
        .dynamic_distance(true)
        .survivor_criteria(SurvivalCriteria::TopNumber(10))
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.50,
            distance: 3.0,
            species_target: 10
        })
        .stagnation(15, vec![
            Genocide::KeepTop(10)
        ])
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            num == 1000
        })?;
        

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
        let data = ISM::read_data().unwrap();
        ISM {
            inputs: data.0,
            answers: data.1
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
                    vec![(x[0] - inputs_min) / (inputs_max - inputs_min), 1.0]
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


    fn read_data() -> Result<(Vec<Vec<f64>>, Vec<Vec<f64>>), Box<dyn Error>> {
        let mut reader = csv::Reader::from_path("C:\\Users\\Peter\\Desktop\\software\\radiate\\examples\\recurrent-neat\\src\\ism.csv").unwrap();
        let mut inputs = Vec::new();
        let mut answers = Vec::new();
        for result in reader.records() {
            let temp = result.unwrap();
            let val: f64 = temp.get(1).unwrap().parse().unwrap();
            inputs.push(vec![val]);
            answers.push(vec![val]);
        }
        inputs.reverse();
        answers.reverse();
        inputs.pop();
        answers.remove(0);
        Ok((inputs, answers))
    }


    fn show(&self, model: &Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = model.feed_forward(&i).unwrap();
            println!("Input: {:.2?} Answer: {:.2?} Guess: {:.2?}", i[0], o[0], guess);
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
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.feed_forward(&ins) {
                Ok(guess) => total += (guess[0] - outs[0]).powf(2.0),
                Err(_) => panic!("Error in training NEAT")
            }
        }
        1.0 - ((1.0 / self.answers.len() as f64) * total)
    }

}