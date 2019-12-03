
extern crate radiate;
extern crate csv;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;

/// this doesn't work but very close


fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let mut neat_env = NeatEnvironment::new()
        .set_input_size(1)
        .set_output_size(1)
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.75)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.04)
        .set_reactivate(0.2)
        .set_c1(1.0)
        .set_c2(1.0)
        .set_c3(0.003)
        .set_node_types(vec![
            NodeType::Recurrent,
        ])
        .set_activation_functions(vec![
            Activation::Tahn,
            Activation::Relu
        ])
        .start_innov_counter();


    let starting_net = Neat::base(&mut neat_env);
    let (solution, _) = Population::<Neat, NeatEnvironment, ISM>::new()
        .constrain(neat_env)
        .size(150)
        .populate_clone(starting_net)
        .debug(true)
        .dynamic_distance(true)
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
            num == 200
        })?;
        

    let ism = ISM::new(1);
    println!("{:#?}", ism);
    let total = ism.solve(&solution);


    println!("Solution: {:#?}", solution);
    solution.see();
    println!("Time in millis: {}", thread_time.elapsed().as_millis());
    ism.show(&solution);
    println!("Total: {}", total);
    // ism.write_data(&solution);
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
        let mut reader = csv::Reader::from_path("C:\\Users\\Peter\\Desktop\\software\\radiate\\examples\\recurrent-neat\\src\\ism.csv").unwrap();
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


    fn write_data(&self, solution: &Neat) {
        let mut writer = csv::Writer::from_path("C:\\Users\\Peter\\Desktop\\software\\radiate\\examples\\recurrent-neat\\src\\output.csv").unwrap();
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = solution.feed_forward(&i).unwrap();
            writer.write_record(&[i[0].to_string(), o[0].to_string(), guess[0].to_string()]).unwrap();
        }
        writer.flush().unwrap();
    }


    fn show(&self, model: &Neat) {
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

    fn empty() -> Self { ISM::new(1) }

    fn solve(&self, model: &Neat) -> f64 {
        let mut total = 0.0;
        let mut goal = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.feed_forward(&ins) {
                Ok(guess) => total += (guess[0] - outs[0]).powf(2.0),
                Err(_) => panic!("Error in training NEAT")
            }
            // goal += outs[0];
        }
        // goal - total
        1.0 - ((1.0 / self.answers.len() as f64) * total)
    }
    
}



#[derive(Debug)]
struct dumbsimple {
    inputs: Vec<Vec<f64>>,
    outputs: Vec<Vec<f64>>
}

impl dumbsimple {

    fn new() -> Self {
        dumbsimple {
            inputs: vec![
                vec![0.0],
                vec![0.0],
                vec![0.0],
                vec![1.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
            ],
            outputs: vec![
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



    fn show(&self, model: &Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.outputs.iter()) {
            let guess = model.feed_forward(&i).unwrap();
            println!("Input: {:?} Answer: {:?} Guess: {:.2?}", i, o, guess);
        }
    }
}


unsafe impl Send for dumbsimple {}
unsafe impl Sync for dumbsimple {}



impl Problem<Neat> for dumbsimple {

    fn empty() -> Self { dumbsimple::new() }

    fn solve(&self, model: &Neat) -> f64 {
        let mut total = 0.0;
        let mut goal = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.outputs.iter()) {
            match model.feed_forward(&ins) {
                Ok(guess) => total += (guess[0] - outs[0]).powf(2.0),
                Err(_) => panic!("Error in training NEAT")
            }
            // goal += outs[0];
        }
        // 7.0 - total
        1.0 - ((1.0 / self.outputs.len() as f64) * total)
    }
    
}