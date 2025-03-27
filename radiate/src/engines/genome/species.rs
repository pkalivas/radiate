use crate::Score;
use std::fmt::{self, Debug, Formatter};

use super::{Chromosome, Genotype};

#[derive(Clone)]
pub struct Species<C: Chromosome> {
    pub mascot: Genotype<C>,
    pub members: Vec<usize>,
    pub score: Score,
}

impl<C: Chromosome> Species<C> {
    pub fn new(mascot: Genotype<C>, score: Score) -> Self {
        Self {
            mascot,
            members: Vec::new(),
            score,
        }
    }

    pub fn add_member(&mut self, index: usize) {
        self.members.push(index);
    }

    pub fn clear_members(&mut self) {
        self.members.clear();
    }
}

impl<C: Chromosome> Debug for Species<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Species {{ members: {:?}, score: {:?} }}",
            self.members.len(),
            self.score
        )
    }
}
