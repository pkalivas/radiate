
extern crate radiate;

use radiate::models::neat::{
    neatenv::NeatEnvironment,
    neat::Neat,
};
use super::populationdto::NeatPopulationBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainDto {
    pub epochs: i32,
    pub learning_rate: f32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSetDto {
    pub inputs: Vec<Vec<f32>>,
    pub answers: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiateDto {
    pub env: Option<NeatEnvironment>,
    pub train: Option<TrainDto>,
    pub neat: Option<Neat>,
    pub population: Option<NeatPopulationBuilder>,
    pub training_set: Option<TrainingSetDto>,
}

impl RadiateDto {
    
    pub fn new() -> Self {
        RadiateDto {
            env: None,
            train: None,
            neat: None,
            population: None,
            training_set: None,
        }
    }

    pub fn env(mut self, env: NeatEnvironment) -> Self {
        self.env = Some(env);
        self
    }

    pub fn train(mut self, epochs: i32, learning_rate: f32) -> Self {
        self.train = Some(TrainDto { epochs, learning_rate });
        self
    }

    pub fn training_set(mut self, inputs: Vec<Vec<f32>>, answers: Vec<Vec<f32>>) -> Self {
        self.training_set = Some(TrainingSetDto { inputs, answers });
        self
    }

    pub fn neat(mut self, neat: Neat) -> Self {
        self.neat = Some(neat);
        self
    }

    pub fn population(mut self, pop: NeatPopulationBuilder) -> Self {
        self.population = Some(pop);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
