
extern crate rand;
extern crate statrs;

use std::mem;
use std::sync::{Arc};
use std::marker::PhantomData;
use rand::Rng;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use statrs::statistics::{
    Mean, 
    Variance, 
};

use super::generation::{Member, MemberWeak};
use super::genome::{Genome};




/// Species member tuple struct to keep track of members and their fitness scores
/// for each species, this decision was made to favor runtime over memory 
#[derive(Debug, Clone)]
pub struct NicheMember<T>(pub f64, pub MemberWeak<T>);


/// A species is meant to keep track of fitness scores of eachof it's members,
/// and a mascot. The mascot is the representation of the species by a Type 
/// member in the population. It also holds the number of age it's been
/// alive
#[derive(Debug, Clone)]
pub struct Niche<T, E> {
    pub mascot: Member<T>,
    pub members: Vec<NicheMember<T>>,
    pub age: i32,
    pub total_adjusted_fitness: Option<f64>,
    phantom: PhantomData<E>
}




/// Implement the species
impl<T, E> Niche<T, E>
    where
        T: Genome<T, E> + Send + Sync,
        E: Send + Sync
{

    // Create a new species with a mascot (weak member pointer)
    pub fn new(mascot: &Member<T>, mascot_fitness: f64) -> Self {
        Niche {
            mascot: Arc::clone(mascot),
            members: vec![NicheMember(mascot_fitness, Arc::downgrade(mascot))],
            age: 0,
            total_adjusted_fitness: None,
            phantom: PhantomData
        }
    }



    /// Get a biased random member from the species. By summing the fitness scores of the 
    /// members, members with larger fitness scorese are statistically more likely to be picked
    #[inline]
    pub fn get_biased_random_member(&self, r: &mut ThreadRng) -> (f64, Member<T>) {
        // declare a result which will panic! at the end of the function if there 
        // is no member found, then get the species total fitness score
        let total = self.get_total_adjusted_fitness();
        let index = r.gen::<f64>() * total;
        let (mut result, mut curr) = (None, 0.0);
        // go through each member and see if it's adjusted fitness has pushed it over the edge
        for member in self.members.iter() {
            curr += member.0;
            if curr >= index {
                result = Some(member);
                break
            }
        };
        // either unwrap the result, or if the adjusted fitness of the species was all
        // negative, just take the first member. If the fitness of the species is negative,
        // the algorithm essentially preforms a random search for these biased functions 
        // once the fitness is above 0, it will 'catch on' and start producing biased results
        result.or_else(|| Some(&self.members[0]))
            .and_then(|val| {
                Some((val.0, val.1.clone()
                    .upgrade()
                    .unwrap_or_else(|| panic!("Failed to get random species member."))
                ))
            })
            .unwrap()
    }



    /// Get the top performing member from the species by their 
    /// associated fitness score. If None is returned meaning there is 
    /// no members in the species, panic!
    pub fn fittest(&self) -> Member<T> {
        let mut top: Option<&NicheMember<T>> = None;
        for i in self.members.iter() {
            if top.is_none() || i.0 > top.unwrap().0 {
                top = Some(i);
            }
        }

        match top {
            Some(t) => Arc::clone(&t.1.upgrade().unwrap()),
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
                self.mascot = Arc::clone(&member.1
                    .upgrade()
                    .unwrap_or_else(|| panic!("Cannot set a nonexistent type to a mascot"))
                );
                self.members = Vec::new();
            }, 
            None => panic!("Failed to get new mascot")
        }
    }



    // for species sizes which are large and populations holding mutliple species,
    // it makes sense to just calcuate this once then retreive the the value 
    // instead of calculate it every time it's needed. Its a quick and simple operation
    pub fn calculate_total_adjusted_fitness(&mut self) {
        let length = self.members.len() as f64;
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
    pub fn get_total_adjusted_fitness(&self) -> f64 {
        match self.total_adjusted_fitness {
            Some(fit) => fit,
            None => panic!("Total adjusted fitness for this species was not set")
        }
    }



    pub fn display_info(&self) {
        let address: u64 = unsafe { mem::transmute(self) };
        let scores = self.members
            .par_iter()
            .map(|x| x.0)
            .collect::<Vec<_>>();
            
        println!("Species: {} gens( {} ) members( {} ) fit( {:.3} ) mean( {:.3} ) var( {:.5} )",
            address,
            self.age,
            self.members.len(),
            self.total_adjusted_fitness.unwrap(),
            scores.mean(),
            if self.members.len() > 1 { scores.variance() } else { 0.0 }
        );
    }

}
