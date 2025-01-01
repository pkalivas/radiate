pub mod alter;
pub mod arithmetic;
pub mod bitflip;
pub mod crossover;
pub mod gaussian;
pub mod intermediate;
pub mod invert;
pub mod mean;
pub mod multipoint;
pub mod mutation;
pub mod pmx;
pub mod scramble;
pub mod shuffle;
pub mod simulated_binary;
pub mod swap;
pub mod uniform;

pub use alter::*;
pub use arithmetic::*;
pub use bitflip::*;
pub use crossover::*;
pub use gaussian::*;
pub use intermediate::*;
pub use invert::*;
pub use mean::*;
pub use multipoint::*;
pub use mutation::*;
pub use pmx::*;
pub use scramble::*;
pub use shuffle::*;
pub use simulated_binary::*;
pub use swap::*;
pub use uniform::*;

use super::{Chromosome, Metric, Population};

pub trait EngineCompoment {
    fn name(&self) -> &'static str;
}

pub enum AlterAction<C: Chromosome> {
    Mutate(Box<dyn MutateAction<C>>),
    Crossover(Box<dyn CrossoverAction<C>>),
}

pub trait EngineAlterer<C: Chromosome>: EngineCompoment {
    fn rate(&self) -> f32;
    fn to_alter(self) -> AlterAction<C>;
}
