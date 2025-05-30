pub use radiate_engines::*;

#[cfg(feature = "gp")]
pub use radiate_gp::*;

pub mod prelude {
    pub use radiate_engines::{
        BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, Ecosystem, Epoch,
        FloatChromosome, FloatGene, Front, Gene, Generation, GeneticEngine, GeneticEngineBuilder,
        IntChromosome, IntGene, MetricSet, MultiObjectiveGeneration, Objective, Optimize,
        Phenotype, Population, Score, Species, log_ctx,
    };
    #[cfg(feature = "gp")]
    pub use radiate_gp::*;
}
