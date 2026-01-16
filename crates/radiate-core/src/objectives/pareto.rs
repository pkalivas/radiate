//! Multi-objective optimization utilities, including Pareto front calculation,
//! non-dominated sorting, crowding distance, and entropy measures.
//! These are essential for evolutionary algorithms that need to handle
//! multiple conflicting objectives.

use crate::objectives::{Objective, Optimize};
use std::collections::HashMap;

/// A small constant to avoid division by zero and ensure non-zero weights.
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
    let n = population.len();
    if n == 0 {
        return Vec::new();
    }

    let mut dominated_counts = vec![0usize; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let a = &population[i];
            let b = &population[j];

            if dominance(a, b, objective) {
                dominated_counts[j] += 1;
            } else if dominance(b, a, objective) {
                dominated_counts[i] += 1;
            }
        }
    }

    let mut nd = Vec::new();
    for i in 0..n {
        if dominated_counts[i] == 0 {
            nd.push(i);
        }
    }

    nd
}

/// Rank the population based on the NSGA-II algorithm. This assigns a rank to each
/// individual in the population based on their dominance relationships with other
/// individuals in the population. The result is a vector of ranks, where the rank
/// of the individual at index `i` is `ranks[i]`.

#[inline]
pub fn rank<T: AsRef<[f32]>>(population: &[T], objective: &Objective) -> Vec<usize> {
    let n = population.len();
    if n == 0 {
        return Vec::new();
    }

    let mut dominated_counts = vec![0usize; n];
    let mut dominates: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut current_front: Vec<usize> = Vec::new();

    // Build dominates lists + dominated counts in one pass (no NxN matrix).
    for i in 0..n {
        for j in (i + 1)..n {
            let a = &population[i];
            let b = &population[j];

            if dominance(a, b, objective) {
                dominates[i].push(j);
                dominated_counts[j] += 1;
            } else if dominance(b, a, objective) {
                dominates[j].push(i);
                dominated_counts[i] += 1;
            }
        }
    }

    // First front
    for i in 0..n {
        if dominated_counts[i] == 0 {
            current_front.push(i);
        }
    }

    let mut ranks = vec![0usize; n];
    let mut front_idx = 0usize;

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
    score_a: &T,
    score_b: &T,
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

/// Calculate the Shannon entropy of a set of scores in multi-dimensional space.
/// The scores are discretized into a grid of bins, and the entropy is computed
/// based on the distribution of scores across these bins. Higher entropy indicates
/// a more diverse set of scores. This can be interpreted as a measure of how well
/// the solutions are spread out in the objective space.
///
/// It works by:
/// 1. Determining the min and max values for each objective dimension.
/// 2. Mapping each score to a discrete bin index based on its normalized position
///    within the min-max range for each dimension.
/// 3. Counting the number of scores in each bin (cell).
/// 4. Calculating the probabilities of each occupied bin and computing the
///    Shannon entropy using these probabilities.
/// 5. Optionally normalizing the entropy by the maximum possible entropy given
///    the number of occupied bins and total scores.
#[inline]
pub fn entropy<S>(scores: &[S], bins_per_dim: usize) -> f32
where
    S: AsRef<[f32]>,
{
    let len = scores.len();
    if len == 0 || bins_per_dim == 0 {
        return 0.0;
    }

    let num_objs = scores[0].as_ref().len();
    if num_objs == 0 {
        return 0.0;
    }

    let mut mins = vec![f32::INFINITY; num_objs];
    let mut maxs = vec![f32::NEG_INFINITY; num_objs];

    for score in scores {
        let values = score.as_ref();
        for dim in 0..num_objs {
            let x = values[dim];

            if x < mins[dim] {
                mins[dim] = x;
            }

            if x > maxs[dim] {
                maxs[dim] = x;
            }
        }
    }

    for dim in 0..num_objs {
        if (maxs[dim] - mins[dim]).abs() < 1e-12 {
            maxs[dim] = mins[dim] + 1.0;
        }
    }

    let mut cell_counts = HashMap::new();

    for score in scores {
        let values = score.as_ref();
        let mut cell = Vec::with_capacity(num_objs);

        for dim in 0..num_objs {
            let norm = (values[dim] - mins[dim]) / (maxs[dim] - mins[dim]); // in [0,1]
            let mut idx = (norm * bins_per_dim as f32).floor() as i32;

            if idx < 0 {
                idx = 0;
            }

            if idx >= bins_per_dim as i32 {
                idx = bins_per_dim as i32 - 1;
            }

            cell.push(idx as u8);
        }

        *cell_counts.entry(cell).or_insert(0) += 1;
    }

    let n_f = len as f32;
    let mut h = 0.0_f32;
    for (_, count) in cell_counts.iter() {
        let p = *count as f32 / n_f;
        if p > 0.0 {
            h -= p * p.ln();
        }
    }

    // Max entropy if all visited cells have equal probabilities.
    // Upper bound by log(min(number_of_cells, n)):
    let k = cell_counts.len().min(len);
    if k > 1 { h / (k as f32).ln() } else { 0.0 }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn obj_min2() -> Objective {
        Objective::Multi(vec![Optimize::Minimize, Optimize::Minimize])
    }

    fn obj_max2() -> Objective {
        Objective::Multi(vec![Optimize::Maximize, Optimize::Maximize])
    }

    // ---- crowding_distance ----

    #[test]
    fn crowding_distance_empty() {
        let scores: Vec<Vec<f32>> = vec![];
        let d = crowding_distance(&scores);
        assert!(d.is_empty());
    }

    #[test]
    fn crowding_distance_zero_dims() {
        let scores = vec![vec![], vec![]];
        let d = crowding_distance(&scores);
        assert_eq!(d, vec![0.0, 0.0]);
    }

    #[test]
    fn crowding_distance_two_points_are_infinite() {
        let scores = vec![vec![0.0f32, 0.0], vec![1.0, 1.0]];
        let d = crowding_distance(&scores);
        assert!(d[0].is_infinite());
        assert!(d[1].is_infinite());
    }

    #[test]
    fn crowding_distance_known_values_1d() {
        // 1D case: interior points should get normalized neighbor span.
        // scores: 0, 1, 2, 3  => range = 3
        // interior at 1: (2-0)/3 = 2/3
        // interior at 2: (3-1)/3 = 2/3
        let scores = vec![vec![0.0f32], vec![1.0], vec![2.0], vec![3.0]];
        let d = crowding_distance(&scores);

        assert!(d[0].is_infinite());
        assert!(d[3].is_infinite());

        // allow tiny float error
        assert!((d[1] - (2.0 / 3.0)).abs() < EPSILON, "d[1] = {}", d[1]);
        assert!((d[2] - (2.0 / 3.0)).abs() < EPSILON, "d[2] = {}", d[2]);
    }

    #[test]
    fn crowding_distance_invariant_under_affine_transform_per_dim() {
        // Because each dim is normalized by (max - min), scaling + shifting a dim should not change
        // crowding distances.
        let a = vec![vec![0.0f32], vec![2.0], vec![4.0], vec![7.0], vec![10.0]];
        let b = a.iter().map(|v| vec![v[0] * 3.0 + 5.0]).collect::<Vec<_>>();

        let da = crowding_distance(&a);
        let db = crowding_distance(&b);

        assert_eq!(da.len(), db.len());
        for i in 0..da.len() {
            if da[i].is_infinite() {
                assert!(db[i].is_infinite());
            } else {
                assert!(
                    (da[i] - db[i]).abs() < 1e-6,
                    "i={}: {} vs {}",
                    i,
                    da[i],
                    db[i]
                );
            }
        }
    }

    #[test]
    fn crowding_distance_constant_dim_contributes_nothing() {
        // Second dimension is constant -> should not affect distances (range == 0 -> skipped).
        let scores = vec![
            vec![0.0f32, 5.0],
            vec![1.0, 5.0],
            vec![2.0, 5.0],
            vec![3.0, 5.0],
        ];
        let d = crowding_distance(&scores);

        assert!(d[0].is_infinite());
        assert!(d[3].is_infinite());

        assert!((d[1] - (2.0 / 3.0)).abs() < EPSILON);
        assert!((d[2] - (2.0 / 3.0)).abs() < EPSILON);
    }

    // ---- dominance ----

    #[test]
    fn dominance_minimization_basic() {
        let a = vec![1.0f32, 2.0];
        let b = vec![2.0f32, 3.0];
        assert!(dominance(&a, &b, &obj_min2()));
        assert!(!dominance(&b, &a, &obj_min2()));
    }

    #[test]
    fn dominance_maximization_basic() {
        let a = vec![5.0f32, 5.0];
        let b = vec![4.0f32, 5.0];

        // maximize: a dominates b (>= in all, > in at least one)
        assert!(dominance(&a, &b, &obj_max2()));
        assert!(!dominance(&b, &a, &obj_max2()));
    }

    #[test]
    fn dominance_equal_scores_is_false() {
        let a = vec![1.0f32, 2.0];
        let b = vec![1.0f32, 2.0];
        assert!(!dominance(&a, &b, &obj_min2()));
        assert!(!dominance(&b, &a, &obj_min2()));
    }

    #[test]
    fn dominance_tradeoff_neither_dominates() {
        let a = vec![1.0f32, 10.0];
        let b = vec![2.0f32, 9.0];
        assert!(!dominance(&a, &b, &obj_min2()));
        assert!(!dominance(&b, &a, &obj_min2()));
    }

    // ---- pareto_front / non_dominated ----

    #[test]
    fn non_dominated_matches_pareto_front_indices() {
        let scores = vec![
            vec![1.0f32, 1.0], // ND
            vec![2.0f32, 2.0], // dominated by [1,1]
            vec![1.0f32, 3.0], // tradeoff with [3,1], ND vs many
            vec![3.0f32, 1.0], // tradeoff with [1,3], ND
            vec![4.0f32, 4.0], // dominated
        ];

        let nd_idx = non_dominated(&scores, &obj_min2());
        let front = pareto_front(&scores, &obj_min2());

        let mut nd_vals = nd_idx
            .iter()
            .map(|&i| scores[i].clone())
            .collect::<Vec<_>>();
        nd_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut front_sorted = front.clone();
        front_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(nd_vals, front_sorted);
    }

    #[test]
    fn pareto_front_contains_all_points_when_no_dominance() {
        // All points trade off -> all non-dominated
        let scores = vec![
            vec![0.0f32, 10.0],
            vec![1.0f32, 9.0],
            vec![2.0f32, 8.0],
            vec![3.0f32, 7.0],
        ];
        let front = pareto_front(&scores, &obj_min2());

        assert_eq!(front.len(), scores.len());
    }

    // ---- rank ----

    #[test]
    fn rank_two_points_dominated_order() {
        // For minimization, [0,0] should be in front 0; [1,1] in front 1.
        let scores = vec![vec![0.0f32, 0.0], vec![1.0f32, 1.0]];
        let r = rank(&scores, &obj_min2());

        assert_eq!(r.len(), 2);
        assert_eq!(r[0], 0, "best point should be rank 0");
        assert_eq!(r[1], 1, "dominated point should be rank 1");
    }

    #[test]
    fn rank_front_partition_expected() {
        // Construct 3 fronts for minimization:
        // F0: A=[0,0]
        // F1: B=[1,0], C=[0,1] (both dominated by A, but neither dominates the other)
        // F2: D=[2,2] (dominated by B and C)
        let a = vec![0.0f32, 0.0];
        let b = vec![1.0f32, 0.0];
        let c = vec![0.0f32, 1.0];
        let d = vec![2.0f32, 2.0];

        let scores = vec![a, b, c, d];
        let r = rank(&scores, &obj_min2());

        assert_eq!(r[0], 0, "A should be in front 0");
        assert_eq!(r[1], 1, "B should be in front 1");
        assert_eq!(r[2], 1, "C should be in front 1");
        assert_eq!(r[3], 2, "D should be in front 2");
    }

    // ---- weights ----

    #[test]
    fn weights_are_positive_and_finite_and_length_matches() {
        let scores = vec![
            vec![0.0f32, 0.0],
            vec![1.0f32, 0.0],
            vec![0.0f32, 1.0],
            vec![2.0f32, 2.0],
        ];

        let w = weights(&scores, &obj_min2());
        assert_eq!(w.len(), scores.len());
        for (i, x) in w.iter().enumerate() {
            assert!(*x > 0.0, "w[{}] should be > 0, got {}", i, x);
            assert!(x.is_finite(), "w[{}] should be finite, got {}", i, x);
        }
    }

    #[test]
    fn weights_prefer_better_front_over_dominated_point() {
        let scores = vec![
            vec![0.0f32, 0.0], // nondominated
            vec![1.0f32, 1.0], // dominated
        ];

        let w = weights(&scores, &obj_min2());
        assert!(w[0] > w[1], "nondominated point should get higher weight");
    }

    // ---- entropy ----

    #[test]
    fn entropy_empty_or_bins_zero() {
        let scores: Vec<Vec<f32>> = vec![];
        assert_eq!(entropy(&scores, 10), 0.0);

        let scores = vec![vec![1.0f32, 2.0]];
        assert_eq!(entropy(&scores, 0), 0.0);
    }

    #[test]
    fn entropy_single_cell_is_zero() {
        // All identical -> all land in one cell -> normalized entropy 0
        let scores = vec![vec![1.0f32, 1.0], vec![1.0f32, 1.0], vec![1.0f32, 1.0]];
        let h = entropy(&scores, 10);

        assert!(h.abs() < 1e-6, "entropy should be ~0, got {}", h);
    }

    #[test]
    fn rank_empty_is_empty() {
        let scores: Vec<Vec<f32>> = vec![];
        let r = rank(&scores, &obj_min2());
        assert!(r.is_empty());
    }

    #[test]
    fn rank_single_is_front0() {
        let scores = vec![vec![1.0f32, 2.0]];
        let r = rank(&scores, &obj_min2());
        assert_eq!(r, vec![0]);
    }

    #[test]
    fn rank_duplicate_points_same_front() {
        // Two identical best points should both be rank 0; dominated point should be later.
        let scores = vec![
            vec![0.0f32, 0.0],
            vec![0.0f32, 0.0], // duplicate
            vec![1.0f32, 1.0], // dominated by both
        ];
        let r = rank(&scores, &obj_min2());

        assert_eq!(r[0], 0);
        assert_eq!(r[1], 0);
        assert_eq!(r[2], 1);
    }

    #[test]
    fn rank_all_nondominated_all_front0() {
        // Tradeoff curve: none dominates another under minimization
        let scores = vec![
            vec![0.0f32, 10.0],
            vec![1.0f32, 9.0],
            vec![2.0f32, 8.0],
            vec![3.0f32, 7.0],
            vec![4.0f32, 6.0],
        ];
        let r = rank(&scores, &obj_min2());
        assert!(
            r.iter().all(|&x| x == 0),
            "expected all rank 0, got {:?}",
            r
        );
    }

    #[test]
    fn rank_strict_chain_increasing_fronts() {
        // Strict dominance chain for minimization:
        // [0,0] dominates [1,1] dominates [2,2] dominates [3,3]
        let scores = vec![
            vec![0.0f32, 0.0],
            vec![1.0f32, 1.0],
            vec![2.0f32, 2.0],
            vec![3.0f32, 3.0],
        ];
        let r = rank(&scores, &obj_min2());
        assert_eq!(r, vec![0, 1, 2, 3]);
    }

    #[test]
    fn rank_matches_iterative_non_dominated_peel() {
        // Property-style test:
        // If we repeatedly peel off the non-dominated set, the peel number should equal rank.
        fn peel_ranks(scores: &[Vec<f32>], objective: &Objective) -> Vec<usize> {
            let mut remaining: Vec<usize> = (0..scores.len()).collect();
            let mut out = vec![usize::MAX; scores.len()];
            let mut front = 0usize;

            while !remaining.is_empty() {
                let subset = remaining.iter().map(|&i| &scores[i]).collect::<Vec<_>>();
                let nd_local = non_dominated(&subset, objective); // indices into subset

                for &k in &nd_local {
                    let global_i = remaining[k];
                    out[global_i] = front;
                }

                // remove the ND points from remaining (in descending order of local indices)
                let mut nd_local_sorted = nd_local;
                nd_local_sorted.sort_unstable_by(|a, b| b.cmp(a));
                for k in nd_local_sorted {
                    remaining.remove(k);
                }

                front += 1;
            }

            out
        }

        let scores = vec![
            vec![0.0f32, 0.0], // F0
            vec![0.0f32, 1.0], // F1
            vec![1.0f32, 0.0], // F1
            vec![1.0f32, 1.0], // F2
            vec![2.0f32, 2.0], // F3
            vec![0.5f32, 0.5], // F2 (dominated by [0,0], not by [0,1] or [1,0])
        ];

        let r = rank(&scores, &obj_min2());
        let p = peel_ranks(&scores, &obj_min2());

        assert_eq!(
            r, p,
            "rank() should match iterative peel ranks\nrank={:?}\npeel={:?}",
            r, p
        );
    }
}
