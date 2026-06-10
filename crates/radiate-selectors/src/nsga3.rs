use radiate_core::{Chromosome, Objective, Optimize, Phenotype, Select, pareto};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

const EPS: f32 = 1e-12;

#[derive(Debug, Clone)]
pub struct NSGA3Selector {
    ref_dirs: Arc<Mutex<Vec<Vec<f32>>>>,
    partitions: usize,
}

impl NSGA3Selector {
    pub fn new(partitions: usize) -> Self {
        Self {
            ref_dirs: Arc::new(Mutex::new(Vec::new())),
            partitions,
        }
    }

    pub fn partitions(&self) -> usize {
        self.partitions
    }

    fn reference_dirs(&self, dims: usize) -> Vec<Vec<f32>> {
        let mut dirs = self.ref_dirs.lock().unwrap();

        if dirs.is_empty() {
            *dirs = pareto::das_dennis(dims, self.partitions);
        }

        dirs.clone()
    }
}

impl<C: Chromosome + Clone> Select<C> for NSGA3Selector {
    fn name(&self) -> &'static str {
        "selector.nsga3"
    }

    fn select(
        &self,
        population: &[Phenotype<C>],
        objective: &Objective,
        count: usize,
    ) -> Vec<usize> {
        if population.is_empty() || count == 0 {
            return Vec::new();
        }

        let scores = population
            .iter()
            .filter_map(|p| p.score())
            .map(|score| to_minimization_space(score.as_ref(), objective))
            .collect::<Vec<_>>();

        let min_objective = minimization_objective(objective.dims());
        let ranks = pareto::rank(&scores, &min_objective);
        let fronts = fronts_from_ranks(&ranks);

        let ref_dirs = self.reference_dirs(objective.dims());

        let mut selected = Vec::with_capacity(count);
        let mut front_idx = 0usize;

        while front_idx < fronts.len() && selected.len() + fronts[front_idx].len() <= count {
            selected.extend_from_slice(&fronts[front_idx]);
            front_idx += 1;
        }

        if selected.len() < count && front_idx < fronts.len() {
            let remaining = count - selected.len();

            selected.extend(niching_fill(
                &scores,
                &ref_dirs,
                &selected,
                &fronts[front_idx],
                remaining,
            ));
        }

        selected.into_iter().take(count).collect()
    }
}

#[inline]
fn minimization_objective(dims: usize) -> Objective {
    Objective::Multi(vec![Optimize::Minimize; dims])
}

#[inline]
pub fn to_minimization_space(score: &[f32], objective: &Objective) -> Vec<f32> {
    match objective {
        Objective::Single(opt) => {
            if *opt == Optimize::Minimize {
                score.to_vec()
            } else {
                score.iter().map(|&x| -x).collect()
            }
        }
        Objective::Multi(opts) => score
            .iter()
            .zip(opts.iter())
            .map(|(&x, opt)| if *opt == Optimize::Minimize { x } else { -x })
            .collect(),
    }
}

#[inline]
pub fn fronts_from_ranks(ranks: &[usize]) -> Vec<Vec<usize>> {
    if ranks.is_empty() {
        return Vec::new();
    }

    let max_rank = *ranks.iter().max().unwrap_or(&0);
    let mut fronts = vec![Vec::<usize>::new(); max_rank + 1];

    for (idx, &rank) in ranks.iter().enumerate() {
        fronts[rank].push(idx);
    }

    while fronts.last().is_some_and(|front| front.is_empty()) {
        fronts.pop();
    }

    fronts
}

#[derive(Debug, Clone)]
pub struct ObjectiveBounds {
    ideal: Vec<f32>,
    nadir: Vec<f32>,
}

impl ObjectiveBounds {
    pub fn from_scores(scores: &[Vec<f32>]) -> Self {
        if scores.is_empty() {
            return Self {
                ideal: Vec::new(),
                nadir: Vec::new(),
            };
        }

        let dims = scores[0].len();
        let mut ideal = vec![f32::INFINITY; dims];
        let mut nadir = vec![f32::NEG_INFINITY; dims];

        for score in scores {
            for dim in 0..dims {
                ideal[dim] = ideal[dim].min(score[dim]);
                nadir[dim] = nadir[dim].max(score[dim]);
            }
        }

        Self { ideal, nadir }
    }

    pub fn normalize(&self, score: &[f32]) -> Vec<f32> {
        score
            .iter()
            .enumerate()
            .map(|(dim, &value)| {
                let den = self.nadir[dim] - self.ideal[dim];

                if !den.is_finite() || den.abs() <= EPS {
                    0.0
                } else {
                    (value - self.ideal[dim]) / den
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Association {
    idx: usize,
    niche: usize,
    distance: f32,
}

/// Given:
/// - `already_selected`: indices chosen from earlier fronts
/// - `last_front`: indices in the partial front
///
/// Returns additional indices from `last_front` using NSGA-III niching.
pub fn niching_fill(
    scores: &[Vec<f32>],
    ref_dirs: &[Vec<f32>],
    already_selected: &[usize],
    last_front: &[usize],
    remaining: usize,
) -> Vec<usize> {
    if remaining == 0 || last_front.is_empty() || ref_dirs.is_empty() {
        return Vec::new();
    }

    let bounds = ObjectiveBounds::from_scores(scores);
    let mut niche_count = vec![0usize; ref_dirs.len()];

    for &idx in already_selected {
        let normalized = bounds.normalize(&scores[idx]);
        let (niche, _) = nearest_reference_direction(&normalized, ref_dirs);
        niche_count[niche] += 1;
    }

    let mut candidates = last_front
        .iter()
        .map(|&idx| {
            let normalized = bounds.normalize(&scores[idx]);
            let (niche, distance) = nearest_reference_direction(&normalized, ref_dirs);

            Association {
                idx,
                niche,
                distance,
            }
        })
        .collect::<Vec<_>>();

    let mut picked = Vec::with_capacity(remaining);

    while picked.len() < remaining && !candidates.is_empty() {
        let niche = least_crowded_candidate_niche(&candidates, &niche_count);
        let candidate_idx = closest_candidate_in_niche(&candidates, niche);

        let selected = candidates.swap_remove(candidate_idx);

        picked.push(selected.idx);
        niche_count[selected.niche] += 1;
    }

    picked
}

#[inline]
fn least_crowded_candidate_niche(candidates: &[Association], niche_count: &[usize]) -> usize {
    candidates
        .iter()
        .map(|candidate| candidate.niche)
        .min_by_key(|&niche| niche_count[niche])
        .unwrap()
}

#[inline]
fn closest_candidate_in_niche(candidates: &[Association], niche: usize) -> usize {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.niche == niche)
        .min_by(|(_, a), (_, b)| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap()
}

#[inline]
pub fn nearest_reference_direction(point: &[f32], ref_dirs: &[Vec<f32>]) -> (usize, f32) {
    let mut best = (0usize, f32::INFINITY);

    for (idx, direction) in ref_dirs.iter().enumerate() {
        let direction_norm = dot(direction, direction);

        if direction_norm <= EPS || !direction_norm.is_finite() {
            continue;
        }

        let projection = dot(point, direction) / direction_norm;
        let distance = perpendicular_distance(point, direction, projection);

        if distance < best.1 {
            best = (idx, distance);
        }
    }

    best
}

#[inline]
fn perpendicular_distance(point: &[f32], direction: &[f32], projection: f32) -> f32 {
    point
        .iter()
        .zip(direction)
        .map(|(&p, &d)| {
            let diff = p - projection * d;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

#[inline]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(&x, &y)| x * y).sum()
}
