use crate::objectives::{pareto, Objective};
use crate::Score;
use itertools::Itertools;
use std::cmp;

#[derive(Debug, Clone, PartialEq)]
pub struct Front {
    scores: Vec<Score>,
    max_size: usize,
    min_size: usize,
    objective: Objective,
}

impl Front {
    pub fn new(max_size: usize, min_size: usize, objective: Objective) -> Self {
        Front {
            scores: Vec::new(),
            max_size,
            min_size,
            objective,
        }
    }

    pub fn scores(&self) -> &Vec<Score> {
        &self.scores
    }

    pub fn update_front(&mut self, scores: &[Score]) {
        for score in scores {
            self.add(score);
        }

        if self.scores.len() > self.max_size {
            self.filter();
        }
    }

    fn add(&mut self, score: &Score) {
        let mut to_remove = Vec::new();
        let mut is_dominated = false;
        let mut remove_dups = false;

        for existing_score in &self.scores {
            if pareto::dominance(score, existing_score, &self.objective) {
                to_remove.push(existing_score.clone());
            } else if pareto::dominance(existing_score, score, &self.objective) || existing_score == score {
                is_dominated = true;
                remove_dups = true;
                break;
            }
        }
        
        if remove_dups {
            self.scores.retain(|x| !to_remove.contains(x));
        }

        if !is_dominated {
            self.scores.retain(|x| !to_remove.contains(x));
            self.scores.push(score.clone());
        }
    }

    fn filter(&mut self) {
        let crowding_distances = pareto::crowding_distance(&self.scores, &self.objective);

        self.scores = crowding_distances
            .iter()
            .enumerate()
            .sorted_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(cmp::Ordering::Equal))
            .take(self.min_size)
            .map(|(i, _)| self.scores[i].clone())
            .collect::<Vec<_>>();
    }
}
