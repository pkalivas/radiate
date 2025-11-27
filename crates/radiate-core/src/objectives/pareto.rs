use crate::objectives::{Objective, Optimize};

const EPSILON: f32 = 1e-6;

/// Calculate the crowding distance for each score in a population.
///
/// The crowding distance is a measure of how close a score is to its neighbors
/// in the objective space. Scores with a higher crowding distance are more
/// desirable because they are more spread out. This is useful for selecting
/// diverse solutions in a multi-objective optimization problem and is a
/// key component of the NSGA-II algorithm.
///
/// For each objective dimension:
/// - Sort individuals by that objective
/// - Boundary points get +∞ distance (always preferred)
/// - Interior points get normalized distance contribution:
///
/// ```text
/// (f_{i+1} - f_{i-1}) / (f_max - f_min)
/// ```
#[inline]
pub fn crowding_distance<T: AsRef<[f32]>>(scores: &[T]) -> Vec<f32> {
    let n = scores.len();
    if n == 0 {
        return Vec::new();
    }

    let m = scores[0].as_ref().len();
    if m == 0 {
        return vec![0.0; n];
    }

    let mut result = vec![0.0f32; n];
    let mut indices: Vec<usize> = (0..n).collect();

    for dim in 0..m {
        indices.sort_unstable_by(|&i, &j| {
            scores[i].as_ref()[dim]
                .partial_cmp(&scores[j].as_ref()[dim])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let min = scores[indices[0]].as_ref()[dim];
        let max = scores[indices[n - 1]].as_ref()[dim];
        let range = max - min;

        if !range.is_finite() || range == 0.0 {
            continue;
        }

        // Boundary points get infinite distance so they’re always preserved
        result[indices[0]] = f32::INFINITY;
        result[indices[n - 1]] = f32::INFINITY;

        // Interior points: normalized distance
        for k in 1..(n - 1) {
            let prev = scores[indices[k - 1]].as_ref()[dim];
            let next = scores[indices[k + 1]].as_ref()[dim];
            let contrib = (next - prev).abs() / range;
            result[indices[k]] += contrib;
        }
    }

    result
}

#[inline]
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
#[inline]
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

/// Combine NSGA-II rank and crowding distance into a single weight in (0, 1].
///
/// - Lower rank (better front) => higher weight
/// - Higher crowding distance  => higher weight
///
/// This weight vector combines both rank and crowding distance to prioritize
/// individuals that are both in better fronts and more diverse within those fronts. Selection
/// algorithms not specifically designed for multi-objective optimization can use these weights
/// as fitness values to guide selection towards a well-distributed Pareto front.
///
/// It follows the approach outlined in the paper [A Fast and Elitist Multiobjective Genetic
/// Algorithm: NSGA-II](https://sci2s.ugr.es/sites/default/files/files/Teaching/OtherPostGraduateCourses/Metaheuristicas/Deb_NSGAII.pdf) by
/// K. Deb, A. Pratap, S. Agarwal, and T. Meyarivan
/// pp. 182-197, Apr. 2002, doi: 10.1109/4235.996017.
///
/// We follow these steps:
/// 1. Compute ranks using the `rank` function (lower is better)
/// 2. Compute crowding distances using the `crowding_distance` function (higher is better).
/// 3. Normalize ranks to [0, 1], where 1 = best front.
/// 4. Normalize crowding distances to [0, 1], where 1 = most isolated.
/// 5. Combine the two normalized values multiplicatively to get the final weight.
#[inline]
pub fn weights<T: AsRef<[f32]>>(scores: &[T], objective: &Objective) -> Vec<f32> {
    let n = scores.len();
    if n == 0 {
        return Vec::new();
    }

    let ranks = rank(scores, objective);
    let distances = crowding_distance(scores);

    let max_rank = *ranks.iter().max().unwrap_or(&0) as f32;

    let rank_weight = ranks
        .iter()
        .map(|r| {
            if max_rank == 0.0 {
                1.0
            } else {
                1.0 - (*r as f32 / max_rank)
            }
        })
        .collect::<Vec<f32>>();

    let finite_max = distances
        .iter()
        .cloned()
        .filter(|d| d.is_finite())
        .fold(0.0f32, f32::max);

    let crowd_weight = distances
        .iter()
        .map(|d| {
            if !d.is_finite() || finite_max == 0.0 {
                1.0
            } else {
                *d / finite_max
            }
        })
        .collect::<Vec<f32>>();

    rank_weight
        .into_iter()
        .zip(crowd_weight.into_iter())
        .map(|(r, c)| (r + EPSILON).max(0.0) * (c + EPSILON).max(0.0))
        .collect()
}

// Determine if one score dominates another score. A score `a` dominates a score `b`
// if it is better in every objective and at least one objective is strictly better.
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
