use crate::{Chromosome, Population};

#[derive(Clone, Debug, PartialEq)]
pub enum Objective {
    Single(Optimize),
    Multi(Vec<Optimize>),
}

impl Objective {
    pub fn sort<C: Chromosome>(&self, population: &mut Population<C>) {
        match self {
            Objective::Single(opt) => opt.sort(population),
            Objective::Multi(_) => population.sort_by(|a, b| {
                let a = a.score().as_ref().unwrap();
                let b = b.score().as_ref().unwrap();
                self.dominance_cmp(&a.values, &b.values)
            }),
        }
    }

    fn dominance_cmp<T>(&self, a: &[T], b: &[T]) -> std::cmp::Ordering
    where
        T: PartialOrd,
    {
        match self {
            Objective::Single(opt) => {
                if opt.is_better(&a[0], &b[0]) {
                    std::cmp::Ordering::Less
                } else if opt.is_better(&b[0], &a[0]) {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            Objective::Multi(opts) => {
                for ((a, b), opt) in a.iter().zip(b.iter()).zip(opts) {
                    if opt.is_better(a, b) {
                        return std::cmp::Ordering::Less;
                    } else if opt.is_better(b, a) {
                        return std::cmp::Ordering::Greater;
                    }
                }
                std::cmp::Ordering::Equal
            }
        }
    }

    pub fn is_better<T>(&self, a: &T, b: &T) -> bool
    where
        T: PartialOrd,
    {
        match self {
            Objective::Single(opt) => opt.is_better(a, b),
            Objective::Multi(opts) => {
                for &opt in opts {
                    if !opt.is_better(a, b) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Optimize {
    Minimize,
    Maximize,
}

impl Optimize {
    pub fn sort<C: Chromosome>(&self, population: &mut Population<C>) {
        match self {
            Optimize::Minimize => population.sort_by(|a, b| a.partial_cmp(b).unwrap()),
            Optimize::Maximize => population.sort_by(|a, b| b.partial_cmp(a).unwrap()),
        }
    }

    pub fn is_better<T>(&self, a: &T, b: &T) -> bool
    where
        T: PartialOrd,
    {
        match self {
            Optimize::Minimize => a < b,
            Optimize::Maximize => a > b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_is_better() {
        assert!(Optimize::Minimize.is_better(&1, &2));
        assert!(!Optimize::Minimize.is_better(&2, &1));
        assert!(Optimize::Maximize.is_better(&2, &1));
        assert!(!Optimize::Maximize.is_better(&1, &2));
    }
}

// let mut order = phenotype_index_pairs
// .iter()
// .map(|(i, _)| *i)
// .collect::<Vec<_>>();
//
// phenotype_index_pairs.sort_by(|a, b| {
// let a_rank = ranks[b.0];
// let b_rank = ranks[a.0];
// let a_distance = distances[b.0];
// let b_distance = distances[a.0];
//
// if a_rank < b_rank || (a_rank == b_rank && a_distance > b_distance) {
// std::cmp::Ordering::Greater
// } else if b_rank < a_rank || (b_rank == a_rank && b_distance > a_distance) {
// std::cmp::Ordering::Less
// } else {
// std::cmp::Ordering::Equal
// }
// });
//
// *population = Population {
// individuals: phenotype_index_pairs
// .iter()
// .map(|(_, i)| i.clone())
// .collect(),
// is_sorted: true,
// };
