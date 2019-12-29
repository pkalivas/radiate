
extern crate rand;

use std::mem;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;
use rand::prelude::SliceRandom;
use rayon::prelude::*;

use super::generation::{Member, MemberWeak};
use super::genome::{Genome};




/// Species member tuple struct to keep track of members and their fitness scores
/// for each species, this decision was made to favor runtime over memory 
#[derive(Debug, Clone)]
pub struct NicheMember<T>(pub f32, pub MemberWeak<T>);


/// A species is meant to keep track of fitness scores of eachof it's members,
/// and a mascot. The mascot is the representation of the species by a Type 
/// member in the population. It also holds the number of age it's been
/// alive
#[derive(Debug, Clone)]
pub struct Niche<T, E> {
    pub mascot: Member<T>,
    pub members: Vec<NicheMember<T>>,
    pub age: i32,
    pub total_adjusted_fitness: Option<f32>,
    phantom: PhantomData<E>
}




/// Implement the species
impl<T, E> Niche<T, E>
    where
        T: Genome<T, E> + Send + Sync + Clone,
        E: Send + Sync
{

    // Create a new species with a mascot (weak member pointer)
    pub fn new(mascot: &Member<T>, mascot_fitness: f32) -> Self {
        Niche {
            mascot: Arc::clone(mascot),
            members: vec![NicheMember(mascot_fitness, Arc::downgrade(mascot))],
            age: 0,
            total_adjusted_fitness: None,
            phantom: PhantomData
        }
    }



    /// Get the top performing member from the species by their 
    /// associated fitness score. If None is returned meaning there is 
    /// no members in the species, panic!
    pub fn fittest(&self) -> (f32, Member<T>) {
        let mut top: Option<&NicheMember<T>> = None;
        for i in self.members.iter() {
            if top.is_none() || i.0 > top.unwrap().0 {
                top = Some(i);
            }
        }

        match top {
            Some(t) => (t.0, Arc::new(RwLock::new((*t.1.upgrade().unwrap()).read().unwrap().clone()))),
            None => panic!("Failed to get top species member.")
        }
    }



    /// Reset the species by getting a new random mascot and incrememnting the 
    /// age by one, then setting the total adjusted species back to None,
    /// and clearing the members vec. Basically starting from scratch again but 
    /// need to incremement a few small things to keep track of the species
    pub fn reset(&mut self) {
        let new_mascot = self.members.choose(&mut rand::thread_rng());
        match new_mascot {
            Some(member) => {
                self.age += 1;
                self.total_adjusted_fitness = None;
                self.mascot = Arc::new(RwLock::new((*member.1.upgrade().unwrap()).read().unwrap().clone()));
                self.members = Vec::new();
            }, 
            None => panic!("Failed to get new mascot")
        }
    }



    // for species sizes which are large and populations holding mutliple species,
    // it makes sense to just calcuate this once then retreive the the value 
    // instead of calculate it every time it's needed. Its a quick and simple operation
    pub fn calculate_total_adjusted_fitness(&mut self) {
        let length = self.members.len() as f32;
        self.total_adjusted_fitness = Some(
            self.members
                .par_iter_mut()
                .map(|x| {
                    x.0 = x.0 / length;
                    x.0
                })
                .sum()
        )
    }



    /// Get the total adjusted fitness score of the species 
    /// by summing up all the fitness scores of each member 
    pub fn get_total_adjusted_fitness(&self) -> f32 {
        match self.total_adjusted_fitness {
            Some(fit) => fit,
            None => panic!("Total adjusted fitness for this species was not set")
        }
    }



    pub fn display_info(&self) {
        let address: u64 = unsafe { mem::transmute(self) };            
        println!("Species: {} gens( {} ) members( {} ) adj fit( {:.3} )",
            address,
            self.age,
            self.members.len(),
            self.total_adjusted_fitness.unwrap(),
        );
    }

}

