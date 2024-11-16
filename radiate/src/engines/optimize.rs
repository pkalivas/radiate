use super::genome::{genes::gene::Gene, population::Population};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Optimize {
    Minimize,
    Maximize,
}

impl Optimize {
    pub fn sort<G, A>(&self, population: &mut Population<G, A>)
    where
        G: Gene<G, A>,
    {
        match self {
            Optimize::Minimize => population.sort_by(|a, b| a.partial_cmp(&b).unwrap()),
            Optimize::Maximize => population.sort_by(|a, b| b.partial_cmp(&a).unwrap()),
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
