/// Provide options for cleaning up the population or applying 
/// some sort of natural selection over the population thorugh time

extern crate rayon;
extern crate rand;  

use std::marker::Sync;
use std::sync::{Arc};
use rayon::prelude::*;
use rand::Rng;
use super::generation::{Generation};
use super::genome::{Genome};
use super::niche::{NicheMember};



/// Definine genocide struct to provide options
/// of what to do when a population is stagnent,
/// ie: how to clean the population
pub enum Genocide {
    KeepTop(usize),
    KillWorst(f32),
    KillRandom(f32),
    KillOldestSpecies(usize)
}




impl Genocide {

    /// match the enum option to what it is, and call it's function to clean the 
    /// population by it.
    pub fn kill<T, E>(&self, generation: &mut Generation<T, E>)
        where 
            T: Genome<T, E> + Send + Sync + Clone,
            E: Send + Sync
    {
        match self {
            Self::KeepTop(gens) => self.keep_top(generation, *gens),
            Self::KillWorst(perc) => self.kill_species_bottom(generation, *perc),
            Self::KillRandom(perc) => self.kill_random_genome(generation, *perc),
            Self::KillOldestSpecies(num) => self.kill_oldest_species(generation, *num)
       }
    }



    /// sort the species in the generation by their age and remove num from them by first
    /// sorting the species then truncating the to keep only num number of species
    fn kill_oldest_species<T, E>(&self, generation: &mut Generation<T, E>, num: usize)
        where 
            T: Genome<T, E> + Send + Sync,
            E: Send + Sync
    {
        if generation.species.len() > num  {
            let to_remove = generation.species.len() - num; 
            generation.species
                .sort_by(|a, b| {
                    let a_age = a.read().unwrap().age;
                    let b_age = b.read().unwrap().age;
                    a_age.partial_cmp(&b_age).unwrap()
                });
            generation.species.truncate(to_remove);
        }
    }



    /// Iterate over each species and remove random members from the species members. This is parallel,
    /// and creates a new vec to put the members who survive into therefore this does require more space
    fn kill_random_genome<T, E>(&self, generation: &mut Generation<T, E>, perc: f32)
        where 
            T: Genome<T, E> + Send + Sync,
            E: Send + Sync
    {
        generation.species
            .par_iter_mut()
            .map_init(|| rand::thread_rng(), |r, spec| {
                let mut new_members = Vec::new();
                for mem in spec.read().unwrap().members.iter() {
                    if r.gen::<f32>() > perc {
                        let solid_member = mem.1.upgrade().unwrap();
                        let copy_member = NicheMember(mem.0, Arc::downgrade(&solid_member));
                        new_members.push(copy_member);
                    }
                }
                if new_members.len() > 0 {
                    spec.write().unwrap().members = new_members;
                }
            })
            .collect::<Vec<_>>();
    }



    /// Kill the bottom prec of each species in the generation. ie: if a species has 10 members 
    /// and the prec is .2, then the bottom two members of this generation will be removed
    /// this function will run in parallel so it sould be fairly quick
    fn kill_species_bottom<T, E>(&self, generation: &mut Generation<T, E>, perc: f32)
        where 
            T: Genome<T, E> + Send + Sync,
            E: Send + Sync
    {
        generation.species 
            .par_iter_mut()
            .map(|spec| {
                let size = spec.read().unwrap().members.len();
                let num_to_remove = size as f32 * perc;
                spec.write().unwrap()
                    .members
                    .sort_by(|a, b| {
                        b.0.partial_cmp(&a.0).unwrap()
                    });
                spec.write().unwrap()
                    .members
                    .truncate(size - num_to_remove as usize);
            })
            .collect::<Vec<_>>();
    }



    /// Keep only the top num of members in the generation by sorting the generation's 
    /// species from best-worst, then truncating the list to keep only the top num
    fn keep_top<T, E>(&self, generation: &mut Generation<T, E>, num: usize)
        where
            T: Genome<T, E> + Send + Sync + Clone,
            E: Send + Sync
    {
        generation.species
            .sort_by(|a, b| {
                let a_fit = a.write().unwrap().get_total_adjusted_fitness();
                let b_fit = b.write().unwrap().get_total_adjusted_fitness();
                b_fit.partial_cmp(&a_fit).unwrap()
            });
        generation.species.truncate(num);
    }

}