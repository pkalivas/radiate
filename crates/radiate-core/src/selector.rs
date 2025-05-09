use crate::Chromosome;
use crate::genome::population::Population;
use crate::objectives::Objective;

use super::random_provider;

/// A trait for selection algorithms. Selection algorithms are used to select
/// individuals from a population to be used in the next generation. The
/// selection process is (most of the time) based on the fitness of the individuals in the
/// population. The selection process can be based on the fitness of the individuals
/// in the population, or it can be based on the individuals themselves.
///
pub trait Select<C: Chromosome>: Send + Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Selector")
    }

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Objective,
        count: usize,
    ) -> Population<C>;
}

/// An iterator that generates random indices based on probabilities.
/// This iterator is used in the RouletteWheel selection algorithm, and
/// Boltzmann selection algorithm. This is essentially the 'roulette wheel'
/// that is spun to select individuals from the population. The probability
/// of selecting an individual is based on the fitness (probability) of the individual.
/// The higher the fitness, the higher the probability of the individual being selected.
pub struct ProbabilityWheelIterator<'a> {
    probabilities: &'a [f32],
    total: f32,
    max_index: usize,
    current: usize,
}

impl<'a> ProbabilityWheelIterator<'a> {
    pub fn new(probabilities: &'a [f32], max_index: usize) -> Self {
        let total = probabilities.iter().sum();
        Self {
            probabilities,
            total,
            max_index,
            current: 0,
        }
    }
}

impl Iterator for ProbabilityWheelIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // In `Radiate` there is a selector for surviving individuals (members who will be selected
        // to be passed on to the next generation without any changes)
        // and a selector for selecting individuals to be used in the
        // next generation. Because of this, we don't select all the individuals
        // in the population, we only select a certain number of individuals.
        // If we have selected all the individuals that this selector is supposed to select, we return None.
        if self.current >= self.max_index {
            return None;
        }

        let mut value = random_provider::random::<f32>() * self.total;
        let mut index = 0;

        // We iterate over the probabilities of the individuals in the population - the 'wheel'
        for (i, &prob) in self.probabilities.iter().enumerate() {
            value -= prob;
            if value <= 0.0 {
                index = i;
                break;
            }
        }

        self.current += 1;
        Some(index)
    }
}
