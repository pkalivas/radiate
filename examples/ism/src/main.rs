
extern crate radiate;
extern crate csv;
extern crate rayon;

use std::error::Error;
use radiate::prelude::*;
use rayon::prelude::*;




fn main() -> Result<(), Box<dyn Error>> {

    // set the number of threads to be used
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    // definie the environment
    let neat_env = NeatEnvironment::new()
        .set_weight_mutate_rate(0.8)
        .set_edit_weights(0.1)
        .set_weight_perturb(1.5)
        .set_new_node_rate(0.03)
        .set_new_edge_rate(0.03)
        .set_reactivate(0.2)
        .set_activation_functions(vec![
            Activation::Sigmoid,
            Activation::Relu,
        ]);
        
    // evolve and train iterations 
    let num_evolve = 50;
    let num_train = 500;

    // problem and solver
    let ism = ISM::new(32);
    let net = Neat::new()
        .input_size(32)
        .batch_size(ism.answers.len())
        .lstm(12, 1, Activation::Sigmoid);
       
    // evolve the solver to fit the problem
    let (mut solution, _) = Population::<Neat, NeatEnvironment, ISM>::new()
        .constrain(neat_env)
        .size(50)
        .populate_clone(net)
        .debug(true)
        .dynamic_distance(true)
        .stagnation(10, vec![Genocide::KillWorst(0.9)])
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.5,
            species_target: 7
        })
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            num == num_evolve
        })?;
            
    // traditional training of neural networks
    solution.train(&ism.inputs, &ism.answers, 0.00001, Loss::Diff, |epoch, loss| {
        let temp = format!("{:.5}", loss).parse::<f32>().unwrap().abs();
        println!("epoch: {:?} loss: {:?}", epoch, temp);
        epoch == num_train || temp < 0.01
    })?;
    
    solution.reset();
    println!("{:?}", ism.solve(&mut solution));

    solution.reset();
    ism.write_data(&mut solution);

     Ok(())
}





#[derive(Debug)]
pub struct ISM {
    min_v: f32,
    max_v: f32,
    inputs: Vec<Vec<f32>>,
    answers: Vec<Vec<f32>>
}



 
impl ISM {

    pub fn new(back: usize) -> Self {
        ISM::read_data(back)
    }
    
    
    fn layer(back: usize, data: Vec<f32>) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
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


    fn minimum(nums: &Vec<Vec<f32>>) -> f32 {
        nums.iter()
            .fold(1000.0, |min, curr| {
                if curr[0] < min {
                    return curr[0]
                }
                min
            })
    }


    fn maximum(nums: &Vec<Vec<f32>>) -> f32 {
        nums.iter()
            .fold(-1000.0, |max, curr| {
                if curr[0] > max {
                    return curr[0]
                }
                max
            })
    }


    fn read_data(back: usize) -> Self {
        let mut reader = csv::Reader::from_path("C:/Users/pkalivas/Desktop/radiate/examples/ism/src/ism_input.csv").unwrap();
        let mut data = Vec::new();
        for result in reader.records() {
            let temp = result.unwrap();
            let val = temp.get(1).unwrap().parse().unwrap();
            data.push(vec![val]);
        }
        let smallest = ISM::minimum(&data);
        let biggest = ISM::maximum(&data);

        data = data.iter()
            .map(|x| vec![(x[0] - smallest) / (biggest - smallest)])
            .collect();
                   
        let mut temp = data.iter().map(|x| x[0]).collect::<Vec<_>>();
        temp.reverse();
        let (o, a) = ISM::layer(back, temp);
        ISM {
            min_v: smallest,
            max_v: biggest,
            inputs: o,
            answers: a
        }
    }



    fn write_data(&self, solution: &mut Neat) {
        let mut writer = csv::Writer::from_path("C:/Users/pkalivas/Desktop/radiate/examples/ism/src/ism.csv").unwrap();
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = solution.forward(i).unwrap();
            writer.write_record(&[
                self.de_norm(i[i.len() - 1]).to_string(), 
                self.de_norm(o[0]).to_string(), 
                self.de_norm(guess[0]).to_string()
            ]).unwrap();
        }
        writer.flush().unwrap();
    }


    fn de_norm(&self, val: f32) -> f32 {
        val * (self.max_v - self.min_v) + self.min_v
    }


    fn show(&self, model: &mut Neat) {
        println!("\n");
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            let guess = model.forward(i).unwrap();
            println!("Input: {:.2?} Answer: {:.2?} Guess: {:.2?}", 
                i.iter()
                .map(|x| format!("{:.2?}", self.de_norm(*x)))
                .collect::<Vec<_>>()
                .join(" "),
                self.de_norm(o[0]), 
                self.de_norm(guess[0])
            );
        }
    }


    fn freestyle(&self, num: usize, model: &mut Neat) {
        let mut guess = Vec::new();
        for (i, o) in self.inputs.iter().zip(self.answers.iter()) {
            guess = model.forward(i).unwrap();
        }

        let mut temp = guess.clone();
        for _ in 0..num {
            guess = model.forward(&temp.to_vec()).unwrap();
            println!("Free Style: input: {:?}, output: {:?}", temp, guess);
            temp = guess.clone();
        }
    }



}


unsafe impl Send for ISM {}
unsafe impl Sync for ISM {}




impl Problem<Neat> for ISM {

    fn empty() -> Self { ISM::new(32) }

    fn solve(&self, model: &mut Neat) -> f32 {
        let mut total = 0.0;
        for (ins, outs) in self.inputs.iter().zip(self.answers.iter()) {
            match model.forward(&ins) {
                Some(guess) => total += (guess[0] - outs[0]).powf(2.0),
                None => panic!("Error in training NEAT")
            }
        }
        model.reset();
        1.0 - ((1.0 / (self.answers.len()) as f32) * total)
    }
    
}