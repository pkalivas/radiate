mod alterer;
mod diversity;
mod selector;

use std::collections::HashMap;

pub use alterer::PyAlterer;
pub use diversity::PyDiversity;
pub use selector::PySelector;

use crate::{PyChromosomeType, PyGeneType};

pub enum InputType {
    Alterer,
    OffspringSelector,
    SurvivorSelector,
    Diversity,
    Objective,
    Limit,
    Subscriber,
    PopulationSize,
    OffspringFraction,
    MaxSpeciesAge,
    MaxPhenotypeAge,
    FrontRange,
    Codec,
    Executor,
    Problem,
    SpeciesThreshold,
}

pub struct EngineParam {
    pub name: String,
    pub input_type: InputType,
    pub gene_type: PyGeneType,
    pub args: HashMap<String, String>,
    pub allowed_genes: Vec<PyGeneType>,
    pub allowed_chromosomes: Vec<PyChromosomeType>,
}
