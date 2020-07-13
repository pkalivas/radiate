
pub mod prelude;
pub mod web;

#[macro_use] extern crate serde_derive;


pub use web::{
    dtos::{
        populationdto::NeatPopulationBuilder,
        radiatedto::{
            RadiateDto,
            TrainDto,
            TrainingSetDto,
        }
    }
};
