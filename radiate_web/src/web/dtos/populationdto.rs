
extern crate radiate;

use radiate::engine::{
    population::Config,
    survival::SurvivalCriteria,
    survival::ParentalCriteria,
    genocide::Genocide
};


#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct NeatPopulationBuilder {
    pub num_evolve: Option<i32>,
    pub target_fitness: Option<f32>,
    pub size: Option<i32>,
    pub dynamic_distance: Option<bool>,
    pub debug_process: Option<bool>,
    pub config: Option<Config>,
    pub stagnation: Option<usize>,
    pub genocide: Option<Vec<Genocide>>,
    pub survivor_criteria: Option<SurvivalCriteria>,
    pub parental_criteria: Option<ParentalCriteria>
}

impl NeatPopulationBuilder {

    pub fn new() -> Self {
        NeatPopulationBuilder {
            num_evolve: None,
            target_fitness: None,
            size: None,
            dynamic_distance: None,
            debug_process: None,
            config: None,
            stagnation: None,
            genocide: None,
            survivor_criteria: Some(SurvivalCriteria::Fittest),
            parental_criteria: Some(ParentalCriteria::BiasedRandom)
        }
    }

    pub fn num_evolve(mut self, num: i32) -> Self{
        self.num_evolve = Some(num);
        self
    }

    pub fn target_fitness(mut self, fitness: f32) -> Self{
        self.target_fitness = Some(fitness);
        self
    }

    pub fn size(mut self, size: i32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn dynamic_distance(mut self, dynamic_distance: bool) -> Self {
        self.dynamic_distance = Some(dynamic_distance);
        self
    }

    pub fn debug_process(mut self, debug_process: bool) -> Self {
        self.debug_process = Some(debug_process);
        self
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn stagnation(mut self, stag: usize) -> Self {
        self.stagnation = Some(stag);
        self
    }

    pub fn genocide(mut self, gen: Vec<Genocide>) -> Self {
        self.genocide = Some(gen);
        self
    }

    pub fn survivor_criteria(mut self, surv: SurvivalCriteria) -> Self {
        self.survivor_criteria = Some(surv);
        self
    }

    pub fn parental_criteria(mut self, par: ParentalCriteria) -> Self {
        self.parental_criteria = Some(par);
        self
    }
    
}
