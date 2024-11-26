use crate::{Objective, Optimize, Score};

pub fn dominance(one: &Score, two: &Score, objective: &Objective) -> bool {
    match objective {
        Objective::Single(opt) => dominance_single(one, two, opt),
        Objective::Multi(opts) => dominance_multi(one, two, opts),
    }
}

pub fn crowding_distance_sort(scores: &mut [Score], objective: &Objective) {
    let len = scores.len();
    let mut sorted = scores.to_vec();
    sorted.sort_by(|a, b| {
        if dominance(a, b, objective) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    let mut max = 0.0;
    let mut min = 0.0;

    for i in 0..sorted.len() {
        if i == 0 || i == len - 1 {
            sorted[i].values.push(0.0);
        } else {
            let prev = &sorted[i - 1];
            let next = &sorted[i + 1];
            let diff = next.values[0] - prev.values[0];
            if diff > max {
                max = diff;
            }
            if diff < min {
                min = diff;
            }
        }
    }

    for i in 0..len {
        let score = &mut scores[i];
        let index = sorted.iter().position(|s| s == score).unwrap();
        let mut distance = 0.0;
        if index > 0 && index < len - 1 {
            let prev = &sorted[index - 1];
            let next = &sorted[index + 1];
            distance = (next.values[0] - prev.values[0] - min) / max;
        }
        score.values.push(distance);
    }
}

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

fn dominance_single(one: &Score, two: &Score, opt: &Optimize) -> bool {
    match opt {
        Optimize::Minimize => one.values[0] < two.values[0],
        Optimize::Maximize => one.values[0] > two.values[0],
    }
}

fn dominance_multi(one: &Score, two: &Score, opts: &[Optimize]) -> bool {
    let mut one_better = false;
    let mut two_better = false;

    for (i, opt) in opts.iter().enumerate() {
        match opt {
            Optimize::Minimize => {
                if one.values[i] < two.values[i] {
                    one_better = true;
                } else if one.values[i] > two.values[i] {
                    two_better = true;
                }
            }
            Optimize::Maximize => {
                if one.values[i] > two.values[i] {
                    one_better = true;
                } else if one.values[i] < two.values[i] {
                    two_better = true;
                }
            }
        }
    }

    one_better && !two_better
}
