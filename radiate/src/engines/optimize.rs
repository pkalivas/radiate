use super::genome::population::Population;
use crate::Chromosome;

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
