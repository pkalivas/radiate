
extern crate rayon;

use std::collections::HashSet;
use std::sync::{Arc, Weak, Mutex};
use rayon::prelude::*;
use super::generation::{Container, Family};
use super::genome::Genome;



//////////////////////////////////////////////////////////////////////////////////////////
/// Note these should not be directly exposed to the user as to avoid confusion with   ///
/// too many knobs to turn to create a population. Instead, provide functions to add   ///
/// them and defaults if they are not added. These are not nessesarily needed options, ///
/// they are add-ons and really only for if you really want to test around with your   ///
/// structure that is evolving, provides users with more options wich is always good   ///
//////////////////////////////////////////////////////////////////////////////////////////



/// Implement a way to pick which way to pick those members who 
/// survice each generation, in other words - pick who gets to stay, 
/// those who do not get to stay 'die off' and are replaced by the children
/// 
/// Fittest - the default option, the top member from each species
/// TopNumber - given a number, keep the top number regardless of species
/// TopPercent - given a percent out of 100, keep the top percent regardless of species
#[derive(Debug, Clone)]
pub enum SurvivalCriteria {
    Fittest,
    TopNumber(usize),
    TopPercent(f32)
}


/// Implement a way to pick parents of children, in other words
/// how is the rest of the population generation after those who 
/// don't survice die out.
/// 
/// BiasedRandom - the default option, statistically pick more fit parents
///                however allow for less fit parents to be picked as well. This is 
///                kinda like putting the members in a species on a curve and randomly 
///                picking from that distribution 
/// OnlySurvivers - those who survive are only allowed to reproduce
/// BestInEachSpecies - only the best in each species are allowed to reproduce
/// MostDifferent - Pick one parent, then find the parent most different from it (structurally) 
///                 and use that as the other parent. Note this could lead to large expansion in population
pub enum PickParents {
    BiasedRandom,
    OnlySurvivers,
    BestInEachSpecies,
    MostDifferent
}



/// Implement the survival enum
impl SurvivalCriteria {

    pub fn pick_survivers<T, E>(&self, members: &mut Vec<Container<T, E>>, families: &Vec<Family<T, E>>) -> Option<Vec<Arc<T>>>
        where
            T: Genome<T, E> + Send + Sync + Clone,
            E: Send + Sync
    {
        match self {
            Self::Fittest => {
                return Some(families.par_iter()
                    .map(|x| x.lock().unwrap().fittest())
                    .collect::<Vec<_>>());
            },
            Self::TopNumber(num) => {
                members.as_mut_slice()
                    .par_sort_by(|a, b| {
                        b.fitness_score.partial_cmp(&a.fitness_score).unwrap()
                    });
                return Some((0..*num)
                    .into_iter()
                    .map(|i| Arc::clone(&members[i].member))
                    .collect());                   
                    
            },
            Self::TopPercent(perc) => {
                let num_to_survive = (members.len() as f32 * perc) as usize;
                members.as_mut_slice()
                .par_sort_by(|a, b| {
                    b.fitness_score.partial_cmp(&a.fitness_score).unwrap()
                });
                return Some((0..num_to_survive)
                    .into_iter()
                    .map(|i| Arc::clone(&members[i].member))
                    .collect()); 
            }
        }
    }

}


