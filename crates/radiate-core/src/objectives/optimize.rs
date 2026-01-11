use super::Scored;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const MIN: &str = "min";
const MAX: &str = "max";

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Objective {
    Single(Optimize),
    Multi(Vec<Optimize>),
}

impl Objective {
    pub fn is_single(&self) -> bool {
        matches!(self, Objective::Single(_))
    }

    pub fn is_multi(&self) -> bool {
        matches!(self, Objective::Multi(_))
    }

    pub fn dims(&self) -> usize {
        match self {
            Objective::Single(_) => 1,
            Objective::Multi(opts) => opts.len(),
        }
    }

    pub fn validate<T: AsRef<[K]>, K>(&self, values: &T) -> bool {
        match self {
            Objective::Single(_) => values.as_ref().len() == 1,
            Objective::Multi(opts) => values.as_ref().len() == opts.len(),
        }
    }

    pub fn cmp<T>(&self, a: &T, b: &T) -> std::cmp::Ordering
    where
        T: PartialOrd,
    {
        match self {
            Objective::Single(opt) => {
                if opt.is_better(a, b) {
                    std::cmp::Ordering::Less
                } else if opt.is_better(b, a) {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            Objective::Multi(opts) => {
                for &opt in opts {
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

    pub fn sort<T: AsMut<[K]>, K: Scored + PartialOrd>(&self, population: &mut T) {
        match self {
            Objective::Single(opt) => opt.sort(population),
            Objective::Multi(_) => population.as_mut().sort_unstable_by(|one, two| {
                if let (Some(score_one), Some(score_two)) = (one.score(), two.score()) {
                    self.dominance_cmp(score_one.as_ref(), score_two.as_ref())
                } else {
                    std::cmp::Ordering::Equal
                }
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

impl AsRef<[Optimize]> for Objective {
    fn as_ref(&self) -> &[Optimize] {
        match self {
            Objective::Single(opt) => std::slice::from_ref(opt),
            Objective::Multi(opts) => opts.as_slice(),
        }
    }
}

impl Default for Objective {
    fn default() -> Self {
        Objective::Single(Optimize::Maximize)
    }
}

impl From<Vec<&str>> for Objective {
    fn from(values: Vec<&str>) -> Self {
        let opts: Vec<Optimize> = values.into_iter().map(|s| Optimize::from(s)).collect();

        if opts.len() == 1 {
            Objective::Single(opts[0])
        } else {
            Objective::Multi(opts)
        }
    }
}

impl Into<Vec<&str>> for Objective {
    fn into(self) -> Vec<&'static str> {
        match self {
            Objective::Single(opt) => vec![opt.into()],
            Objective::Multi(opts) => opts.into_iter().map(|opt| opt.into()).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Optimize {
    Minimize,
    Maximize,
}

impl Optimize {
    pub fn sort<T: AsMut<[K]>, K: PartialOrd>(&self, population: &mut T) {
        match self {
            Optimize::Minimize => population
                .as_mut()
                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            Optimize::Maximize => population
                .as_mut()
                .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)),
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

    pub fn is_minimize(&self) -> bool {
        matches!(self, Optimize::Minimize)
    }

    pub fn is_maximize(&self) -> bool {
        matches!(self, Optimize::Maximize)
    }
}

impl From<&str> for Optimize {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            MIN => Optimize::Minimize,
            MAX => Optimize::Maximize,
            _ => Optimize::Maximize,
        }
    }
}

impl Into<&str> for Optimize {
    fn into(self) -> &'static str {
        match self {
            Optimize::Minimize => MIN,
            Optimize::Maximize => MAX,
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

    #[test]
    fn test_objective_is_better_single() {
        let obj = Objective::Single(Optimize::Minimize);
        assert!(obj.is_better(&1, &2));
        assert!(!obj.is_better(&2, &1));
        let obj = Objective::Single(Optimize::Maximize);
        assert!(obj.is_better(&2, &1));
        assert!(!obj.is_better(&1, &2));
    }
}
