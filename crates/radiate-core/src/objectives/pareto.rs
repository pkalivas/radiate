use crate::objectives::{Objective, Optimize};

/// Calculate the crowding distance for each score in a population.
///
/// The crowding distance is a measure of how close a score is to its neighbors
/// in the objective space. Scores with a higher crowding distance are more
/// desirable because they are more spread out. This is useful for selecting
/// diverse solutions in a multi-objective optimization problem and is a
/// key component of the NSGA-II algorithm.
pub fn crowding_distance<T: AsRef<[f32]>>(scores: &[T], objective: &Objective) -> Vec<f32> {
    let indices = scores
        .iter()
        .enumerate()
        .map(|(i, score)| (score.as_ref(), i))
        .collect::<Vec<(&[f32], usize)>>();

    let mut result = vec![0.0; scores.len()];

    for i in 0..indices[0].0.len() {
        let mut distance_values = indices.clone();
        distance_values.sort_by(|a, b| b.0[i].partial_cmp(&a.0[i]).unwrap());

        let min = indices[distance_values[0].1];
        let max = indices[distance_values[distance_values.len() - 1].1];

        let dm = distance(max.0, min.0, objective.as_ref(), i);

        if dm == 0.0 {
            continue;
        }

        result[min.1] = f32::INFINITY;
        result[max.1] = f32::INFINITY;

        for j in 1..distance_values.len() - 1 {
            let prev = indices[distance_values[j - 1].1];
            let next = indices[distance_values[j + 1].1];
            let dp = distance(next.0, prev.0, objective.as_ref(), i);

            result[distance_values[j].1] += dp / dm;
        }
    }

    result
}

pub fn non_dominated<T: AsRef<[f32]>>(population: &[T], objective: &Objective) -> Vec<usize> {
    let mut dominated_counts = vec![0; population.len()];
    let mut dominates = vec![Vec::new(); population.len()];

    for i in 0..population.len() {
        for j in (i + 1)..population.len() {
            let score_one = &population[i];
            let score_two = &population[j];
            if dominance(score_one, score_two, objective) {
                dominates[i].push(j);
                dominated_counts[j] += 1;
            } else if dominance(score_two, score_one, objective) {
                dominates[j].push(i);
                dominated_counts[i] += 1;
            }
        }
    }

    let mut non_dominated = Vec::new();
    for i in 0..population.len() {
        if dominated_counts[i] == 0 {
            non_dominated.push(i);
        }
    }

    non_dominated
}

/// Rank the population based on the NSGA-II algorithm. This assigns a rank to each
/// individual in the population based on their dominance relationships with other
/// individuals in the population. The result is a vector of ranks, where the rank
/// of the individual at index `i` is `ranks[i]`.
pub fn rank<T: AsRef<[f32]>>(population: &[T], objective: &Objective) -> Vec<usize> {
    let mut dominated_counts = vec![0; population.len()];
    let mut dominates = vec![Vec::new(); population.len()];
    let mut current_front: Vec<usize> = Vec::new();
    let mut dominance_matrix = vec![vec![0; population.len()]; population.len()];

    for i in 0..population.len() {
        for j in (i + 1)..population.len() {
            let score_one = &population[i];
            let score_two = &population[j];
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

pub fn weights<T: AsRef<[f32]>>(scores: &[T], objective: &Objective) -> Vec<f32> {
    let ranks = rank(scores, objective);
    let distances = crowding_distance(scores, objective);

    let rank_weight = ranks
        .iter()
        .map(|r| 1.0 / (*r as f32))
        .collect::<Vec<f32>>();
    let max_crowding = distances.iter().cloned().fold(0.0, f32::max);
    let crowding_weight = distances
        .iter()
        .map(|d| 1.0 - d / max_crowding)
        .collect::<Vec<f32>>();

    rank_weight
        .iter()
        .zip(crowding_weight.iter())
        .map(|(r, c)| r * c)
        .collect::<Vec<f32>>()
}

/// Determine if one score dominates another score. A score `a` dominates a score `b`
/// if it is better in every objective and at least one objective is strictly better.
pub fn dominance<K: PartialOrd, T: AsRef<[K]>>(
    score_a: T,
    score_b: T,
    objective: &Objective,
) -> bool {
    let mut better_in_any = false;

    match objective {
        Objective::Single(opt) => {
            for (a, b) in score_a.as_ref().iter().zip(score_b.as_ref().iter()) {
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
        Objective::Multi(opts) => {
            for ((a, b), opt) in score_a.as_ref().iter().zip(score_b.as_ref()).zip(opts) {
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

    better_in_any
}

/// Calculate the Pareto front of a set of scores. The Pareto front is the set of
/// scores that are not dominated by any other score in the set. This is useful
/// for selecting the best solutions in a multi-objective optimization problem.
pub fn pareto_front<K: PartialOrd, T: AsRef<[K]> + Clone>(
    values: &[T],
    objective: &Objective,
) -> Vec<T> {
    let mut front = Vec::new();
    for score in values {
        let mut dominated = false;
        for other in values {
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
fn distance<K: PartialOrd, T: AsRef<[K]>>(one: T, two: T, opts: &[Optimize], index: usize) -> f32 {
    match opts[index] {
        Optimize::Minimize => {
            if one.as_ref()[index] > two.as_ref()[index] {
                1.0
            } else if one.as_ref()[index] < two.as_ref()[index] {
                -1.0
            } else {
                0.0
            }
        }
        Optimize::Maximize => {
            if one.as_ref()[index] < two.as_ref()[index] {
                1.0
            } else if one.as_ref()[index] > two.as_ref()[index] {
                -1.0
            } else {
                0.0
            }
        }
    }
}
