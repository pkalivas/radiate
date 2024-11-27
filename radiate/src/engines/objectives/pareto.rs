use crate::objectives::{Objective, Optimize};
use crate::{Chromosome, Population, Score};

/// Calculate the crowding distance for each score in a population.
///
/// The crowding distance is a measure of how close a score is to its neighbors
/// in the objective space. Scores with a higher crowding distance are more
/// desirable because they are more spread out. This is useful for selecting
/// diverse solutions in a multi-objective optimization problem and is a
/// key component of the NSGA-II algorithm.
pub fn crowding_distance(scores: &[Score], objective: &Objective) -> Vec<f32> {
    let indices = scores
        .iter()
        .enumerate()
        .map(|(i, score)| (score, i))
        .collect::<Vec<(&Score, usize)>>();

    let mut result = vec![0.0; scores.len()];

    for i in 0..scores[0].values.len() {
        let mut distance_values = indices.clone();
        distance_values.sort_by(|a, b| b.0.values[i].partial_cmp(&a.0.values[i]).unwrap());

        let min = indices[distance_values[0].1];
        let max = indices[distance_values[distance_values.len() - 1].1];

        let dm = distance(max.0, min.0, objective, i);

        if dm == 0.0 {
            continue;
        }

        result[min.1] = f32::INFINITY;
        result[max.1] = f32::INFINITY;

        for j in 1..distance_values.len() - 1 {
            let prev = indices[distance_values[j - 1].1];
            let next = indices[distance_values[j + 1].1];
            let dp = distance(next.0, prev.0, objective, i);

            result[distance_values[j].1] += dp / dm;
        }
    }

    result
}

/// Rank the population based on the NSGA-II algorithm. This assigns a rank to each
/// individual in the population based on their dominance relationships with other
/// individuals in the population. The result is a vector of ranks, where the rank
/// of the individual at index `i` is `ranks[i]`.
pub fn rank<C: Chromosome>(population: &Population<C>, objective: &Objective) -> Vec<usize> {
    let mut dominated_counts = vec![0; population.len()];
    let mut dominates = vec![Vec::new(); population.len()];
    let mut current_front: Vec<usize> = Vec::new();
    let mut dominance_matrix = vec![vec![0; population.len()]; population.len()];

    for i in 0..population.len() {
        for j in (i + 1)..population.len() {
            let score_one = population[i].score_as_ref();
            let score_two = population[j].score_as_ref();
            if dominance(score_one, score_two, objective) {
                dominance_matrix[i][j] = 1;
                dominance_matrix[j][i] = -1;
            } else if dominance(score_two, score_one, objective) {
                dominance_matrix[i][j] = -1;
                dominance_matrix[j][i] = 1;
            }
        }
    }

    for i in 0..population.len() {
        for j in 0..population.len() {
            if i != j {
                if dominance_matrix[i][j] == 1 {
                    dominates[i].push(j);
                } else if dominance_matrix[i][j] == -1 {
                    dominated_counts[i] += 1;
                }
            }
        }

        // If no one dominates this solution, it belongs to the first front
        if dominated_counts[i] == 0 {
            current_front.push(i);
        }
    }

    // Assign ranks based on fronts
    let mut ranks = vec![0; population.len()];
    let mut front_idx = 0;

    while !current_front.is_empty() {
        let mut next_front = Vec::new();

        for &p in &current_front {
            ranks[p] = front_idx;

            for &q in &dominates[p] {
                dominated_counts[q] -= 1;
                if dominated_counts[q] == 0 {
                    next_front.push(q);
                }
            }
        }

        front_idx += 1;
        current_front = next_front;
    }

    ranks
}

/// Determine if one score dominates another score. A score `a` dominates a score `b`
/// if it is better in every objective and at least one objective is strictly better.
pub fn dominance(score_a: &Score, score_b: &Score, objective: &Objective) -> bool {
    let mut better_in_any = false;
    for (a, b) in score_a.values.iter().zip(&score_b.values) {
        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    if a > b {
                        return false;
                    }
                    if a < b {
                        better_in_any = true;
                    }
                } else {
                    if a < b {
                        return false;
                    }
                    if a > b {
                        better_in_any = true;
                    }
                }
            }
            Objective::Multi(opts) => {
                for ((a, b), opt) in score_a.values.iter().zip(&score_b.values).zip(opts) {
                    if opt == &Optimize::Minimize {
                        if a > b {
                            return false;
                        }
                        if a < b {
                            better_in_any = true;
                        }
                    } else {
                        if a < b {
                            return false;
                        }
                        if a > b {
                            better_in_any = true;
                        }
                    }
                }
            }
        }
    }
    better_in_any
}

/// Calculate the Pareto front of a set of scores. The Pareto front is the set of
/// scores that are not dominated by any other score in the set. This is useful
/// for selecting the best solutions in a multi-objective optimization problem.
pub fn pareto_front(scores: &[Score], objective: &Objective) -> Vec<Score> {
    let mut front = Vec::new();
    for score in scores {
        let mut dominated = false;
        for other in scores {
            if dominance(other, score, objective) {
                dominated = true;
                break;
            }
        }
        if !dominated {
            front.push(score.clone());
        }
    }
    front
}

/// Calculate the distance between two scores in the objective space. This is used
/// to calculate the crowding distance for each score in a population.
fn distance(one: &Score, two: &Score, objective: &Objective, index: usize) -> f32 {
    match objective {
        Objective::Single(opt) => distance_single(one, two, opt, index),
        Objective::Multi(opts) => distance_multi(one, two, opts, index),
    }
}

fn distance_single(one: &Score, two: &Score, opt: &Optimize, index: usize) -> f32 {
    match opt {
        Optimize::Minimize => one.values[index] - two.values[index],
        Optimize::Maximize => two.values[index] - one.values[index],
    }
}

fn distance_multi(one: &Score, two: &Score, opts: &[Optimize], index: usize) -> f32 {
    match opts[index] {
        Optimize::Minimize => one.values[index] - two.values[index],
        Optimize::Maximize => two.values[index] - one.values[index],
    }
}
