
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
        .set_weight_perturb(1.7)
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
        .input_size(2)
        .lstm(20, 1);

    
    let num_evolve = 10;
    let (mut solution, _) = Population::<Neat, NeatEnvironment, ISM>::new()
        .constrain(neat_env)
        .size(100)
        .populate_clone(starting_net)
        .debug(true)
        .dynamic_distance(true)
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.5,
            species_target: 4
        })
        .stagnation(15, vec![
            Genocide::KillWorst(0.9)
        ])
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            num == num_evolve
        })?;
        

        let ism = ISM::new(2);
        println!("{:#?}", ism);
        let total = ism.solve(&mut solution);
    
    
        println!("Solution: {:#?}", solution);
        println!("Time in millis: {}", thread_time.elapsed().as_millis());
        ism.show(&mut solution);
        println!("Total: {}", total);
        ism.write_data(&mut solution);
        Ok(())
    }
    


#[derive(Debug)]
pub struct ISM {
    inputs: Vec<Vec<f64>>,
    answers: Vec<Vec<f64>>
}



impl ISM {


    pub fn new(back: usize) -> Self {
        let data = ISM::read_data(back).unwrap();
        ISM {
            inputs: data.0,
            answers: data.1
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


    fn read_data(back: usize) -> Result<(Vec<Vec<f64>>, Vec<Vec<f64>>), Box<dyn Error>> {
        let mut reader = csv::Reader::from_path("C:\\Users\\Peter\\Desktop\\software\\radiate\\examples\\lstm-neat\\src\\ism.csv").unwrap();
        let mut data = Vec::new();
        for result in reader.records() {
            let temp = result.unwrap();
            let val: f64 = temp.get(1).unwrap().parse().unwrap();
            data.push(vec![val]);
        }
        let smallest = ISM::minimum(&data);
        let biggest = ISM::maximum(&data);
        data = data.iter()
            .map(|x| {
                vec![(x[0] - smallest) / (biggest - smallest)]
            })
            .collect();
                           
        let mut temp = data.iter().map(|x| x[0]).collect::<Vec<_>>();
        temp.reverse();
        Ok(ISM::layer(back, temp))
    }


    fn layer(back: usize, data: Vec<f64>) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let mut output = Vec::new();
        let mut answer = Vec::new();
        for i in 0..data.len() - back{
            let mut temp = Vec::with_capacity(back);
            for j in 0..back {
                temp.push(data[i + j]);
            }
            answer.push(vec![data[i + back]]);
            output.push(temp);
        }
        (output, answer)
    }


    fn write_data(&self, solution: &mut Neat) {
        let mut writer = csv::Writer::from_path("C:\\Users\\Peter\\Desktop\\software\\radiate\\examples\\lstm-neat\\src\\output.csv").unwrap();
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = solution.feed_forward(&i).unwrap();
            writer.write_record(&[i[0].to_string(), o[0].to_string(), guess[0].to_string()]).unwrap();
        }
        writer.flush().unwrap();
    }


    fn show(&self, model: &mut Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = model.feed_forward(&i).unwrap();
            println!("Input: {:?} Answer: {:?} Guess: {:.2?}", i, o, guess);
        }
    }
    

    #[allow(dead_code)]
    fn backprop(&self, model: &mut Neat) {
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            model.backprop(i, o, 0.1);
        }
    }

}



unsafe impl Send for ISM {}
unsafe impl Sync for ISM {}



impl Problem<Neat> for ISM {

    fn empty() -> Self { ISM::new(2) }

    fn solve(&self, model: &mut Neat) -> f64 {
        let mut total = 0.0;
        let mut goal = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.feed_forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
            // goal += outs[0];
        }
        // goal - total
        1.0 - ((1.0 / self.answers.len() as f64) * total)
    }
    
}