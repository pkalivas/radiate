use rand::random;

use super::gene::{BoundGene, Gene, NumericGene, Valid};

pub struct FloatGene {
    pub allele: f32,
    pub min: f32,
    pub max: f32,
    pub upper_bound: f32,
    pub lower_bound: f32,
}

impl FloatGene {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            allele: random::<f32>() * (max - min) + min,
            min,
            max,
            upper_bound: f32::MAX,
            lower_bound: f32::MIN,
        }
    }
}

impl Valid for FloatGene {
    fn is_valid(&self) -> bool {
        self.allele >= self.min && self.allele <= self.max
    }
}

impl Gene<FloatGene, f32> for FloatGene {
    fn allele(&self) -> &f32 {
        &self.allele
    }

    fn new_instance(&self) -> FloatGene {
        FloatGene {
            allele: random::<f32>() * (self.max - self.min) + self.min,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn from_allele(&self, allele: &f32) -> FloatGene {
        FloatGene {
            allele: *allele,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

impl BoundGene<FloatGene, f32> for FloatGene {
    fn upper_bound(&self) -> &f32 {
        &self.upper_bound
    }

    fn lower_bound(&self) -> &f32 {
        &self.lower_bound
    }

    fn with_bounds(self, upper_bound: f32, lower_bound: f32) -> FloatGene {
        FloatGene {
            upper_bound,
            lower_bound,
            ..self
        }
    }
}

impl NumericGene<FloatGene, f32> for FloatGene {
    fn add(&self, other: &FloatGene) -> FloatGene {
        FloatGene {
            allele: self.allele + other.allele,
            ..*self
        }
    }

    fn sub(&self, other: &FloatGene) -> FloatGene {
        FloatGene {
            allele: self.allele - other.allele,
            ..*self
        }
    }

    fn mul(&self, other: &FloatGene) -> FloatGene {
        FloatGene {
            allele: self.allele * other.allele,
            ..*self
        }
    }

    fn div(&self, other: &FloatGene) -> FloatGene {
        let other_allele = match other.allele() == &0_f32 {
            true => 1_f32,
            false => *other.allele(),
        };

        FloatGene {
            allele: self.allele / other_allele,
            ..*self
        }
    }

    fn mean(&self, other: &FloatGene) -> FloatGene {
        FloatGene {
            allele: (self.allele + other.allele) / 2_f32,
            ..*self
        }
    }
}

impl Clone for FloatGene {
    fn clone(&self) -> Self {
        FloatGene {
            allele: self.allele,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

impl PartialEq for FloatGene {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele && self.min == other.min && self.max == other.max
    }
}

impl std::fmt::Debug for FloatGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}
