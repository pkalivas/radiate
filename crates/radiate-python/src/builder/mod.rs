mod alter;
mod limit;
mod selector;

pub use alter::*;
pub use limit::*;
pub(crate) use selector::set_selector;
// use radiate::{Chromosome, Epoch, Generation, GeneticEngineBuilder, Optimize};

// pub(crate) fn map_objectives<C, T, E>(
//     builder: GeneticEngineBuilder<C, T>,
//     objectives: Vec<String>,
// ) -> GeneticEngineBuilder<C, T, E>
// where
//     C: Chromosome,
//     T: Clone + Send + Sync,
//     E: Epoch<Chromosome = C>,
// {
//     let mut directions = Vec::new();
//     for obj in objectives.iter() {
//         let obj = obj.to_lowercase();
//         if obj == "min" {
//             directions.push(Optimize::Minimize);
//         } else if obj == "max" {
//             directions.push(Optimize::Maximize);
//         } else {
//             panic!("Unknown objective type");
//         }
//     }

//     return panic!("Multi-objective optimization is not yet supported");

//     // if directions.len() == 1 {
//     //     let new_builder = builder as GeneticEngineBuilder<C, T, Generation<C, T>>;
//     //     match directions[0] {
//     //         Optimize::Minimize => new_builder.minimizing(),
//     //         Optimize::Maximize => new_builder.maximizing(),
//     //     }
//     // } else {
//     //     builder.multi_objective(directions)
//     // }
// }
