
extern crate radiate;

use std::error::Error;
use std::time::Instant;
use radiate::prelude::*;


/// Dumb test:
/// two inputs and one output
/// given two inputs at each step, the desired output 
/// at each step is the output for the column that was 
/// two time steps ago
/// 
/// [0, 0, 1, 1, 0, (1), 0, 0]
/// [0, 1, 0, 0, 1, (1), 0, 1]
/// --------------------------
/// [1, 0, 0, 0, 0, 0, 0, (1)]


fn main() -> Result<(), Box<dyn Error>> {

    let thread_time = Instant::now();
    let mut neat_env = NeatEnvironment::new()
        .set_input_size(3)
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

    // make new network with nothing in it 
    let mut starting_net = Neat::new();

    // create the nodes in the network - to test a recurrent network we don't need any hidden nodes
    let mut input_one = Vertex::new(neat_env.get_mut_counter().next(), Layer::Input, NodeType::Recurrent, Activation::Tahn);
    starting_net.inputs.push(input_one.innov);
    let mut input_two = Vertex::new(neat_env.get_mut_counter().next(), Layer::Input, NodeType::Recurrent, Activation::Tahn);
    starting_net.inputs.push(input_two.innov);
    let mut input_three = Vertex::new(neat_env.get_mut_counter().next(), Layer::Input, NodeType::Recurrent, Activation::Tahn);
    starting_net.inputs.push(input_three.innov);
    let mut output = Vertex::new(neat_env.get_mut_counter().next(), Layer::Output, NodeType::Dense, Activation::Sigmoid);
    starting_net.outputs.push(output.innov);

    // make the edges to connect the vertecies
    let one_output = Edge::new(input_one.innov, output.innov, neat_env.get_mut_counter().next(), 0.5, true);
    input_one.outgoing.push(one_output.innov);
    output.incoming.insert(one_output.innov, None);
    let two_output = Edge::new(input_two.innov, output.innov, neat_env.get_mut_counter().next(), 0.5, true);
    input_two.outgoing.push(two_output.innov);
    output.incoming.insert(two_output.innov, None);
    let three_output = Edge::new(input_three.innov, output.innov, neat_env.get_mut_counter().next(), 0.5, true);
    input_three.outgoing.push(three_output.innov);
    output.incoming.insert(three_output.innov, None);

    starting_net.nodes.insert(input_one.innov, input_one.as_mut_ptr());
    starting_net.nodes.insert(input_two.innov, input_two.as_mut_ptr());
    starting_net.nodes.insert(input_three.innov, input_three.as_mut_ptr());
    starting_net.nodes.insert(output.innov, output.as_mut_ptr());
    starting_net.edges.insert(one_output.innov, one_output);
    starting_net.edges.insert(two_output.innov, two_output);
    starting_net.edges.insert(three_output.innov, three_output);

    println!("Created Network:\n {:#?}", starting_net);


    let starting_net = Neat::base(&mut neat_env);
    let (solution, _) = Population::<Neat, NeatEnvironment, AND>::new()
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
            Genocide::KeepTop(5)
        ])
        .run(|_, fit, num| {
            println!("Generation: {} score: {}", num, fit);
            let diff = 4.0 - fit;
            (diff > 0.0 && diff < 0.01) || num == 1500
        })?;
        

    let and_hist = AND::new();
    let total = and_hist.solve(&solution);

    println!("Solution: {:#?}", solution);
    solution.see();
    println!("Time in millis: {}", thread_time.elapsed().as_millis());
    and_hist.show(&solution);
    println!("Total: {}", total);

    Ok(())
}






/// [0, 0, 1, 1, 0, (1), 0, 0]
/// [0, 1, 0, 0, 1, (1), 0, 1]
/// --------------------------
/// [1, 0, 0, 0, 0, 0, 0, (1)]

#[derive(Debug)]
pub struct AND {
    inputs: Vec<Vec<f64>>,
    answers: Vec<Vec<f64>>
}



impl AND {
    pub fn new() -> Self {
        AND {
            inputs: vec![
                vec![0.0, 0.0, 1.5],
                vec![0.0, 1.0, 1.5],
                vec![1.0, 0.0, 1.5],
                vec![1.0, 0.0, 1.5],
                vec![0.0, 1.0, 1.5],
                vec![1.0, 1.0, 1.5],
                vec![0.0, 0.0, 1.5],
                vec![0.0, 1.0, 1.5],
            ],
            answers: vec![
                vec![1.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
                vec![0.0],
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

}


unsafe impl Send for AND {}
unsafe impl Sync for AND {}




impl Problem<Neat> for AND {

    fn empty() -> Self { AND::new() }

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