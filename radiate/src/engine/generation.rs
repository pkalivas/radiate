extern crate rand;

use std::sync::{Arc, Weak, RwLock};
use rayon::prelude::*;
use super::niche::{Niche, NicheMember};
use super::{
    genome::Genome,
    problem::Problem,
    environment::Envionment,
    population::Config,
    survival::{SurvivalCriteria, ParentalCriteria}
};




/// The member type is meant to represent a holder for 
/// just the type T, where it has an owning reference counter
/// then wrapped in a ref cell to allow for mutable borrowing of the Rc
pub type Member<T> = Arc<RwLock<T>>;
/// the MemberWeak is meant to be a Nonowning member type pointing to the 
/// same memory space but not having the same owning ability of the data
pub type MemberWeak<T> = Weak<RwLock<T>>;

/// A family is a wrapper for a species type which ownes the data it 
/// holds. This is needed as there are many references to a species 
/// throughout the program
pub type Family<T, E> = Arc<RwLock<Niche<T, E>>>;
/// the FamilyWeak is meant to mimic the MemberWeak as it is a nonowning family
/// type which allows for multiple bi-directional pointers to the same Niche 
/// type in the same memory location
pub type FamilyWeak<T, E> = Weak<RwLock<Niche<T, E>>>;




/// A container is a simple container to encapsulate a member (Type T)
/// its fitness score for the current generation, and a weak reference 
/// counting cell to the species it belongs to
#[derive(Debug)]
pub struct Container<T, E>
    where 
        T: Genome<T, E> + Send + Sync,
        E: Send + Sync
{
    pub member: Member<T>,
    pub fitness_score: f32,
    pub species: Option<FamilyWeak<T, E>>
}


impl<T, E> Container<T, E>
    where 
        T: Genome<T, E> + Send + Sync,
        E: Send + Sync
{
    pub fn get_member(&mut self) -> &mut Member<T> {
        &mut self.member
    }

    pub fn update_member(&mut self, new_member: T) {
        *self.member.write().unwrap() = new_member;
    }

    pub fn set_fitness(&mut self, fitness: f32) {
        self.fitness_score = fitness;
    }
}



/// A generation is meant to facilitate the speciation, crossover, and 
/// reproduction of spececies and their types over the course of a single 
/// genertion
#[derive(Debug)]
pub struct Generation<T, E> 
    where
        T: Genome<T, E> + Send + Sync,
        E: Send + Sync
{
    pub members: Vec<Container<T, E>>,
    pub species: Vec<Family<T, E>>,
    pub survival_criteria: SurvivalCriteria,
    pub parental_criteria: ParentalCriteria
}



/// implement the generation
impl<T, E> Generation<T, E> 
    where
        T: Genome<T, E> + Send + Sync + Clone,
        E: Envionment + Sized + Send + Sync
{
    /// Create a new generation
    /// 
    /// This creates a base default generation type with no 
    /// members and no species. It is bland.
    pub fn new() -> Self {
        Generation {
            members: Vec::new(),
            species: Vec::new(),
            survival_criteria: SurvivalCriteria::Fittest,
            parental_criteria: ParentalCriteria::BiasedRandom
        }
    }

    /// passdown the previous generation's members and species to a new generation
    #[inline]
    pub fn pass_down(&self, new_members: Vec<Member<T>>) -> Option<Self> {
        Some(Generation {
            members: new_members
                .into_par_iter()
                .map(|x| {
                    Container {
                        member: Arc::clone(&x),
                        fitness_score: 0.0,
                        species: None
                    }
                })
                .collect(),
            species: self.species
                .par_iter()
                .map(|spec| {
                    spec.write().unwrap().reset();
                    Arc::clone(spec)
                })
                .collect(),
            survival_criteria: self.survival_criteria.clone(),
            parental_criteria: self.parental_criteria.clone()
        })
    }

    /// Get mutable slice of current generation members.
    pub fn members_mut(&mut self) -> &mut [Container<T, E>] {
        &mut self.members
    }

    /// Get mutable member.
    pub fn member_mut(&mut self, idx: usize) -> Option<&mut Container<T, E>> {
        self.members.get_mut(idx)
    }

    /// Get immutable member.
    pub fn member(&self, idx: usize) -> Option<&Container<T, E>> {
        self.members.get(idx)
    }

    /// The optimization function
    #[inline]
    pub fn optimize<P>(&mut self, prob: Arc<RwLock<P>>)
        where P: Problem<T> + Send + Sync
    {
        // concurrently iterate the members and optimize them
        self.members
            .par_iter_mut()
            .for_each_with(prob, |problem, cont| {
                (*cont).fitness_score = problem.read().unwrap().solve(&mut *cont.member.write().unwrap());
            });
    }

    /// Speciation is the process of going through the members in the generation
    /// and assigning them species in which they belong to determined by a specific 
    /// distance between the member and the species mascot.
    #[inline]
    pub fn speciate(&mut self, distance: f32, settings: Arc<RwLock<E>>) {
        // Loop over the members mutably to find a species which this member belongs to
        for cont in self.members.iter_mut() {
            // see if this member belongs to a given species 
            let mem_spec = self.species
                .iter()
                .find(|s| {
                    <T as Genome<T, E>>::distance(&*cont.member.read().unwrap(), &*s.read().unwrap().mascot.read().unwrap(), Arc::clone(&settings)) < distance
                });
            // if the member does belong to an existing species, add the two to each other 
            // otherwise create a new species and add that to the species and the member 
            match mem_spec {
                Some(spec) => {
                    let mut lock_spec = spec.write().unwrap();
                    lock_spec.members.push(NicheMember(cont.fitness_score, Arc::downgrade(&cont.member)));
                    cont.species = Some(Arc::downgrade(spec));
                },
                None => {
                    let new_family = Arc::new(RwLock::new(Niche::new(&cont.member, cont.fitness_score)));
                    cont.species = Some(Arc::downgrade(&new_family));
                    self.species.push(new_family);
                }
            }
        }
        // first filter out all species with have died out.
        // go through and set the total adjusted fitness for each species
        self.species.retain(|x| Arc::weak_count(&x) > 0);
        for i in self.species.iter() {
            i.write().unwrap().calculate_total_adjusted_fitness();
        }
    }

    /// Create the next generation and return a new generation struct with 
    /// new members, and reset species. This is how the generation moves from
    /// one to the next. This function also is the one which runs the crossover
    /// fn from the genome trait, the more effecent that function is, the faster
    /// this function will be.
    #[inline]
    pub fn create_next_generation(&mut self, pop_size: i32, config: Config, env: Arc<RwLock<E>>) -> Option<Self> {   
        // generating new members in a biased way using rayon to parallize it
        // then crossover to fill the rest of the generation 
        let mut new_members = self.survival_criteria.pick_survivers(&mut self.members, &self.species)?;
        let children = (new_members.len() as i32..pop_size)
            .into_par_iter()
            .map(|_|{
                // select two random species to crossover, with a chance of inbreeding then cross them over
                let (one, two) = self.parental_criteria.pick_parents(config.inbreed_rate, &self.species).unwrap();
                let child = if one.0 > two.0 {
                    <T as Genome<T, E>>::crossover(&*one.1.read().unwrap(), &*two.1.read().unwrap(), Arc::clone(&env), config.crossover_rate).unwrap()
                } else {
                    <T as Genome<T, E>>::crossover(&*two.1.read().unwrap(), &*one.1.read().unwrap(), Arc::clone(&env), config.crossover_rate).unwrap()
                };
                Arc::new(RwLock::new(child))
            })
            .collect::<Vec<_>>();
        // reset the species and passdown the new members to a new generation
        new_members.extend(children);
        self.pass_down(new_members)
    }

    /// get the top member of the generations
    #[inline] 
    pub fn best_member(&self) -> Option<(f32, Arc<T>)> {
        let mut top: Option<&Container<T, E>> = None;
        for i in self.members.iter() {
            if top.is_none() || i.fitness_score > top?.fitness_score {
                top = Some(i);
            }
        }
        // return the best member of the generation
        match top {
            Some(t) => Some((t.fitness_score, Arc::new((*t.member).read().unwrap().clone()))),
            None => None
        }
    }
}
