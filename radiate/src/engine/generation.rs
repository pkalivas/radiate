
extern crate rand;

use std::sync::{Arc, Weak, Mutex};
use rand::Rng;
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use super::niche::{Niche, NicheMember};
use super::{
    genome::{Genome},
    problem::{Problem},
    environment::{Envionment},
    population::{Config}
};




/// The member type is meant to represent a holder for 
/// just the type T, where it has an owning reference counter
/// then wrapped in a ref cell to allow for mutable borrowing of the Rc
pub type Member<T> = Arc<T>;
/// the MemberWeak is meant to be a Nonowning member type pointing to the 
/// same memory space but not having the same owning ability of the data
pub type MemberWeak<T> = Weak<T>;

/// A family is a wrapper for a species type which ownes the data it 
/// holds. This is needed as there are many references to a species 
/// throughout the program
type Family<T, E> = Arc<Mutex<Niche<T, E>>>;
/// the FamilyWeak is meant to mimic the MemberWeak as it is a nonowning family
/// type which allows for multiple bi-directional pointers to the same Niche 
/// type in the same memory location
type FamilyWeak<T, E> = Weak<Mutex<Niche<T, E>>>;



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
    pub fitness_score: f64,
    pub species: Option<FamilyWeak<T, E>>
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
}



/// implement the generation
impl<T, E> Generation<T, E> 
    where
        T: Genome<T, E> + Send + Sync,
        E: Send + Sync
{

    /// Create a new generation
    /// 
    /// This creates a base default generation type with no 
    /// members and no species. It is bland.
    pub fn new() -> Self {
        Generation {
            members: Vec::new(),
            species: Vec::new(),
        }
    }



    /// passdown the previous generation's members and species to a new species
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
                    spec.lock().unwrap().reset();
                    Arc::clone(spec)
                })
                .collect()
        })
    }



    /// The optimization function
    #[inline]
    pub fn optimize<P>(&mut self, prob: Arc<P>) -> Option<(f64, Arc<T>)>
        where 
            T: Clone,
            P: Problem<T> + Send + Sync
     {
         // concurrently iterate the members and optimize them
         self.members
            .par_iter_mut()
            .for_each_with(prob, |problem, cont| {
                (*cont).fitness_score = problem.solve(&*cont.member);
            });

        // return the top member from the optimization as a tuple (f64, Arc<T>)
        self.best_member()
     }



    /// Speciation is the process of going through the members in the generation
    /// and assigning them species in which they belong to determined by a specific 
    /// distance between the member and the species mascot.
    #[inline]
    pub fn speciate(&mut self, distance: f64, settings: &Arc<Mutex<E>>) {
        // Loop over the members mutably to find a species which this member belongs to
        for cont in self.members.iter_mut() {
            // see if this member belongs to a given species 
            let mem_spec = self.species.iter()
                .find(|s| {
                    let lock_spec = s.lock().unwrap();
                    <T as Genome<T, E>>::distance(&*cont.member, &*lock_spec.mascot, settings) < distance
                });
            // if the member does belong to an existing species, add the two to each other 
            // otherwise create a new species and add that to the species and the member 
            match mem_spec {
                Some(spec) => {
                    let mut lock_spec = spec.lock().unwrap();
                    lock_spec.members.push(NicheMember(cont.fitness_score, Arc::downgrade(&cont.member)));
                    cont.species = Some(Arc::downgrade(spec));
                },
                None => {
                    let new_family = Arc::new(Mutex::new(Niche::new(&cont.member, cont.fitness_score)));
                    cont.species = Some(Arc::downgrade(&new_family));
                    self.species.push(new_family);
                }
            }
        }
        // first filter out all species with have died out.
        // go through and set the total adjusted fitness for each species
        self.species.retain(|x| Arc::weak_count(&x) > 0);
        for i in self.species.iter() {
            i.lock().unwrap().calculate_total_adjusted_fitness();
        }
    }



    /// Create the next generation and return a new generation struct with 
    /// new members, and reset species. This is how the generation moves from
    /// one to the next. This function also is the one which runs the crossover
    /// fn from the genome trait, the more effecent that function is, the faster
    /// this function will be.
    #[inline]
    pub fn create_next_generation(&mut self, pop_size: i32, config: Config, settings: &Arc<Mutex<E>>) -> Option<Self>
        where 
            T: Sized + Clone,
            E: Envionment + Sized + Send + Sync
    {   
        // generating new members in a biased way using rayon to parallize it
        let mut new_members = self.species.par_iter()
            .map(|x| x.lock().unwrap().fittest())
            .collect::<Vec<_>>();
        // crossover to fill the rest of the generation 
        new_members.extend((new_members.len() as i32..pop_size)
            .into_par_iter()
            .map(|_|{
                // select two random species to crossover, with a chance of inbreeding then cross them over
                let (one, two) = self.pick_parents(config.inbreed_rate);
                let child = if one.0 > two.0 {
                    <T as Genome<T, E>>::crossover(&*one.1, &*two.1, settings, config.crossover_rate).unwrap()
                } else {
                    <T as Genome<T, E>>::crossover(&*two.1, &*one.1, settings, config.crossover_rate).unwrap()
                };
                Arc::new(child)
            })
            .collect::<Vec<_>>()
        );
        // reset the species and passdown the new members to a new generation
        self.pass_down(new_members)
    }



    /// pick two parents to breed a child - these use biased random ways of picking 
    /// parents and returns a tuple of tuples where the f64 is the parent's fitness,
    /// and the type is the parent itself
    #[inline]
    fn pick_parents(&self, inbreed_rate: f32) -> ((f64, Arc<T>), (f64, Arc<T>)) {
        let mut r = rand::thread_rng();
        let (species_one, species_two);
        // get two species to pick from taking into account an inbreeding rate - an inbreed can happen without this 
        if r.gen::<f32>() < inbreed_rate {
            let temp = self.get_biased_random_species(&mut r).unwrap();
            species_one = Arc::clone(&temp);
            species_two = temp;
        } else {
            species_one = self.get_biased_random_species(&mut r).unwrap();
            species_two = self.get_biased_random_species(&mut r).unwrap();
        }
        // get two parents from the species, again the parent may be the same 
        let parent_one = species_one.lock().unwrap().get_biased_random_member(&mut r);
        let parent_two = species_two.lock().unwrap().get_biased_random_member(&mut r);
        // return the parent tuples
        (parent_one, parent_two)
    }



    /// get a biased random species from the population to get members from
    /// this gets a random species by getting the total adjusted fitness of the 
    /// entire population then finding a random number inside (0, total populatin fitness)
    /// then summing the individual species until they hit that random numer 
    ///
    /// Statistically this allows for species with larger adjusted fitnesses to 
    /// have a greater change of being picked for breeding
    #[inline]
    fn get_biased_random_species(&self, r: &mut ThreadRng) -> Option<Family<T, E>> {
        // set a result option to none, this will panic! if the result is still none
        // at the end of the function. Then get the total poopulation fitness
        let mut result = None;
        let total = self.species.iter()
            .fold(0.0, |sum, curr| {
                sum + (*curr).lock().unwrap().get_total_adjusted_fitness()
            });

        // iterate through the species until the iterative sum is at or above the selected
        // random adjusted fitness level
        let mut curr = 0.0;
        let index = r.gen::<f64>() * total;
        for i in self.species.iter() {
            curr += i.lock().ok()?.get_total_adjusted_fitness();
            if curr >= index {
                result = Some(Arc::clone(i));
                break
            }
        }

        // either return the result, or panic!
        result.or_else(|| Some(Arc::clone(self.species.first()?)))
    }


    
    /// get the top member of the generations
    #[inline] 
    pub fn best_member(&self) -> Option<(f64, Arc<T>)>
        where T: Genome<T, E> + Clone 
    {
        let mut top: Option<&Container<T, E>> = None;
        for i in self.members.iter() {
            if top.is_none() || i.fitness_score > top?.fitness_score {
                top = Some(i);
            }
        }

        match top {
            Some(t) => Some((t.fitness_score, Arc::clone(&t.member))),
            None => None
        }
    }


}
