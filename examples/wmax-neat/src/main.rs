extern crate radiate;

use std::error::Error;
use radiate::prelude::*;



fn main() -> Result<(), Box<dyn Error>> {

    // let mut neat_env = default_neat_env();
    // let starting_net = Neat::new().connect(2, 1, neat_env.get_mut_counter());
    // let (solution, _) = Population::<Neat, NeatEnvironment, NeatWeightMax>::new()
    //     .constrain(neat_env)
    //     .size(250)
    //     .populate_clone(starting_net)
    //     .debug(true)
    //     .configure(Config {
    //         inbreed_rate: 0.001,
    //         crossover_rate: 0.75,
    //         distance: 3.0,
    //         species_target: 4
    //     })
    //     .run(|_, fit, num| {
    //         println!("Generation: {} score: {}", num, fit);
    //         (100.0 - fit).abs() > 0.0 && (100.0 - fit).abs() < 0.0001
    //     })?;
    
    // println!("Solution");
    // println!("{:#?}", solution);
    

    Ok(())
}




// struct NeatWeightMax;

// impl Problem<Neat> for NeatWeightMax {
//     fn empty() -> Self { NeatWeightMax }

//     fn solve(&self, model: &Neat) -> f64 {
//         let mut total = 0.0;
//         for edge in model.edges.values() {
//             total += edge.weight;
//         }
//         100.0 - (100.0 - total).abs()
//     }
// }




 