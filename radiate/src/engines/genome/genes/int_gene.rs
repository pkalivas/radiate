use rand::distributions::Standard;

use crate::RandomProvider;

use super::{
    gene::{BoundGene, Gene, NumericGene, Valid},
    Integer,
};

pub struct IntGene<T: Integer<T>>
where
    Standard: rand::distributions::Distribution<T>,
{
    allele: T,
    min: T,
    max: T,
    upper_bound: T,
    lower_bound: T,
}

impl<T: Integer<T>> IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    pub fn new(allele: T) -> Self {
        Self {
            allele,
            min: T::MIN,
            max: T::MAX,
            upper_bound: T::MAX,
            lower_bound: T::MIN,
        }
    }

    pub fn from_min_max(min: T, max: T) -> Self {
        let (min, max) = if min > max { (max, min) } else { (min, max) };

        Self {
            allele: RandomProvider::gen_range(min..max),
            min,
            max,
            upper_bound: T::MAX,
            lower_bound: T::MIN,
        }
    }
}

impl<T: Integer<T>> Gene<IntGene<T>, T> for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn allele(&self) -> &T {
        &self.allele
    }

    fn new_instance(&self) -> IntGene<T> {
        IntGene {
            allele: RandomProvider::gen_range(self.min..self.max),
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

impl<T: Integer<T>> Valid for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn is_valid(&self) -> bool {
        self.allele >= self.min && self.allele <= self.max
    }
}

impl<T: Integer<T>> BoundGene<IntGene<T>, T> for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
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

impl<T: Integer<T>> NumericGene<IntGene<T>, T> for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
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

impl<T: Integer<T>> Clone for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
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

impl<T: Integer<T>> PartialEq for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl<T: Integer<T>> std::fmt::Debug for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}

impl Into<IntGene<i8>> for i8 {
    fn into(self) -> IntGene<i8> {
        IntGene::new(self)
    }
}

impl Into<IntGene<i16>> for i16 {
    fn into(self) -> IntGene<i16> {
        IntGene::new(self)
    }
}

impl Into<IntGene<i32>> for i32 {
    fn into(self) -> IntGene<i32> {
        IntGene::new(self)
    }
}

impl Into<IntGene<i64>> for i64 {
    fn into(self) -> IntGene<i64> {
        IntGene::new(self)
    }
}

impl Into<IntGene<i128>> for i128 {
    fn into(self) -> IntGene<i128> {
        IntGene::new(self)
    }
}

impl From<IntGene<i8>> for i8 {
    fn from(gene: IntGene<i8>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i16>> for i16 {
    fn from(gene: IntGene<i16>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i32>> for i32 {
    fn from(gene: IntGene<i32>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i64>> for i64 {
    fn from(gene: IntGene<i64>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i128>> for i128 {
    fn from(gene: IntGene<i128>) -> Self {
        gene.allele
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new() {
        let gene = IntGene::from_min_max(0, 10);
        assert!(gene.allele >= 0 && gene.allele <= 10);
    }

    #[test]
    fn test_new_instance() {
        let gene = IntGene::from_min_max(0, 10);
        let new_gene = gene.new_instance();
        assert!(new_gene.allele >= 0 && new_gene.allele <= 10);
    }

    #[test]
    fn test_from_allele() {
        let gene = IntGene::new(5);
        let new_gene = gene.from_allele(&5);
        assert_eq!(new_gene.allele, 5);
    }

    #[test]
    fn test_is_valid() {
        let gene = IntGene::from_min_max(0, 10);
        assert!(gene.is_valid());
    }

    #[test]
    fn test_upper_bound() {
        let gene = IntGene::from_min_max(0, 10).with_bounds(10, 0);
        assert_eq!(*gene.upper_bound(), 10);
    }

    #[test]
    fn test_lower_bound() {
        let gene = IntGene::from_min_max(0, 10).with_bounds(10, 0);
        assert_eq!(*gene.lower_bound(), 0);
    }

    #[test]
    fn test_add() {
        let gene = IntGene::new(5);
        let other = IntGene::new(5);
        let new_gene = gene.add(&other);
        assert_eq!(new_gene.allele, 10);
    }

    #[test]
    fn test_sub() {
        let gene = IntGene::new(5);
        let other = IntGene::new(5);
        let new_gene = gene.sub(&other);
        assert_eq!(new_gene.allele, 0);
    }

    #[test]
    fn test_mul() {
        let gene = IntGene::new(5);
        let other = IntGene::new(5);
        let new_gene = gene.mul(&other);
        assert_eq!(new_gene.allele, 25);
    }

    #[test]
    fn test_div() {
        let gene = IntGene::new(5);
        let other = IntGene::new(5);
        let new_gene = gene.div(&other);
        assert_eq!(new_gene.allele, 1);
    }

    #[test]
    fn test_mean() {
        let gene = IntGene::new(5);
        let other = IntGene::new(5);
        let new_gene = gene.mean(&other);
        assert_eq!(new_gene.allele, 5);
    }

    #[test]
    fn test_into() {
        let gene: IntGene<i32> = 5.into();
        assert_eq!(gene.allele, 5);
    }

    #[test]
    fn test_from() {
        let gene = IntGene::new(5);
        let i: i32 = gene.into();
        assert_eq!(i, 5);
    }
}
