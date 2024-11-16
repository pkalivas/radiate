use rand::Rng;

use super::{
    gene::{BoundGene, Gene, NumericGene, Valid},
    Integer,
};

pub struct IntGene<T: Integer<T>> {
    allele: T,
    min: T,
    max: T,
    upper_bound: T,
    lower_bound: T,
}

impl<T: Integer<T>> IntGene<T> {
    pub fn new(min: T, max: T) -> Self {
        let (min, max) = if min > max { (max, min) } else { (min, max) };
        let mut rand = rand::thread_rng();

        Self {
            allele: rand.gen_range(min..max),
            min,
            max,
            upper_bound: T::MAX,
            lower_bound: T::MIN,
        }
    }
}

impl<T: Integer<T>> Gene<IntGene<T>, T> for IntGene<T> {
    fn allele(&self) -> &T {
        &self.allele
    }

    fn new_instance(&self) -> IntGene<T> {
        let mut rand = rand::thread_rng();
        IntGene {
            allele: rand.gen_range(self.min..self.max),
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn from_allele(&self, allele: &T) -> IntGene<T> {
        IntGene {
            allele: *allele,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

impl<T: Integer<T>> Valid for IntGene<T> {
    fn is_valid(&self) -> bool {
        self.allele >= self.min && self.allele <= self.max
    }
}

impl<T: Integer<T>> BoundGene<IntGene<T>, T> for IntGene<T> {
    fn upper_bound(&self) -> &T {
        &self.upper_bound
    }

    fn lower_bound(&self) -> &T {
        &self.lower_bound
    }

    fn with_bounds(self, upper_bound: T, lower_bound: T) -> IntGene<T> {
        IntGene {
            upper_bound,
            lower_bound,
            ..self
        }
    }
}

impl<T: Integer<T>> NumericGene<IntGene<T>, T> for IntGene<T> {
    fn add(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self.allele + other.allele,
            ..*self
        }
    }

    fn sub(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self.allele - other.allele,
            ..*self
        }
    }

    fn mul(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self.allele * other.allele,
            ..*self
        }
    }

    fn div(&self, other: &IntGene<T>) -> IntGene<T> {
        let other_allele = match other.allele() == &T::from_i32(0) {
            true => T::from_i32(1),
            false => *other.allele(),
        };

        IntGene {
            allele: self.allele / other_allele,
            ..*self
        }
    }

    fn mean(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: (self.allele + other.allele) / T::from_i32(2),
            ..*self
        }
    }
}

impl<T: Integer<T>> Clone for IntGene<T> {
    fn clone(&self) -> Self {
        IntGene {
            allele: self.allele,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

impl<T: Integer<T>> PartialEq for IntGene<T> {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl<T: Integer<T>> std::fmt::Debug for IntGene<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}
