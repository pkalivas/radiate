use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

pub trait Scored {
    fn score(&self) -> Option<&Score>;
}

/// A score is a value that can be used to compare the fitness of two individuals and represents
/// the 'fitness' of an individual within the genetic algorithm.
/// The score can be a single value or multiple values, depending on the problem being solved.
/// For ease of use the `Score` struct provides methods
/// to convert the score to a single value, an integer, a string, or a vector of `f32` values.
///
/// Note: The reason it is a Vec is for multi-objective optimization problems. This allows for multiple
/// fitness values to be returned from the fitness function.
#[derive(Clone, PartialEq, Default)]
pub struct Score {
    pub values: Arc<[f32]>,
}

impl Score {
    pub fn from_any(value: &dyn std::any::Any) -> Self {
        if let Some(value) = value.downcast_ref::<f32>() {
            Score::from_f32(*value)
        } else if let Some(value) = value.downcast_ref::<i32>() {
            Score::from_int(*value)
        } else if let Some(value) = value.downcast_ref::<usize>() {
            Score::from_usize(*value)
        } else if let Some(value) = value.downcast_ref::<String>() {
            Score::from_string(value)
        } else if let Some(value) = value.downcast_ref::<Score>() {
            value.clone()
        } else if let Some(value) = value.downcast_ref::<Vec<f32>>() {
            Score::from_vec(value.clone())
        } else {
            panic!("Invalid type for Score")
        }
    }

    pub fn from_vec(values: Vec<f32>) -> Self {
        for value in &values {
            if value.is_nan() {
                panic!("Score value cannot be NaN")
            }
        }

        Score {
            values: Arc::from(values),
        }
    }

    pub fn from_f32(value: f32) -> Self {
        if value.is_nan() {
            panic!("Score value cannot be NaN")
        }

        Score {
            values: Arc::from(vec![value]),
        }
    }

    pub fn from_int(value: i32) -> Self {
        Score {
            values: Arc::from(vec![value as f32]),
        }
    }

    pub fn from_usize(value: usize) -> Self {
        Score {
            values: Arc::from(vec![value as f32]),
        }
    }

    pub fn from_string(value: &str) -> Self {
        Score {
            values: Arc::from(vec![
                value.parse::<f32>().expect("Failed to parse string to f32"),
            ]),
        }
    }

    pub fn as_f32(&self) -> f32 {
        self.values.get(0).cloned().unwrap_or(f32::NAN)
    }

    pub fn as_i32(&self) -> i32 {
        self.values[0] as i32
    }

    pub fn as_string(&self) -> String {
        self.values[0].to_string()
    }

    pub fn as_usize(&self) -> usize {
        self.values[0] as usize
    }
}

impl AsRef<[f32]> for Score {
    fn as_ref(&self) -> &[f32] {
        &self.values
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.values.partial_cmp(&other.values)
    }
}

impl Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.values)
    }
}

impl Hash for Score {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut hash: usize = 0;

        for value in self.values.iter() {
            let value_hash = value.to_bits();
            hash = hash.wrapping_add(value_hash as usize);
        }

        hash.hash(state);
    }
}

impl From<f32> for Score {
    fn from(value: f32) -> Self {
        Score::from_f32(value)
    }
}

impl From<i32> for Score {
    fn from(value: i32) -> Self {
        Score::from_int(value)
    }
}

impl From<usize> for Score {
    fn from(value: usize) -> Self {
        Score::from_usize(value)
    }
}

impl From<String> for Score {
    fn from(value: String) -> Self {
        Score::from_string(&value)
    }
}

impl From<&str> for Score {
    fn from(value: &str) -> Self {
        Score::from_string(value)
    }
}

impl From<Vec<f32>> for Score {
    fn from(value: Vec<f32>) -> Self {
        Score::from_vec(value)
    }
}

impl From<Vec<i32>> for Score {
    fn from(value: Vec<i32>) -> Self {
        Score::from_vec(value.into_iter().map(|v| v as f32).collect())
    }
}

impl From<Vec<usize>> for Score {
    fn from(value: Vec<usize>) -> Self {
        Score::from_vec(value.into_iter().map(|v| v as f32).collect())
    }
}

impl From<Vec<String>> for Score {
    fn from(value: Vec<String>) -> Self {
        Score::from_vec(
            value
                .into_iter()
                .map(|v| v.parse::<f32>().unwrap())
                .collect(),
        )
    }
}

impl From<Vec<&str>> for Score {
    fn from(value: Vec<&str>) -> Self {
        Score::from_vec(
            value
                .into_iter()
                .map(|v| v.parse::<f32>().unwrap())
                .collect(),
        )
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.values.is_empty() {
            return other;
        }
        let values = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a + b)
            .collect();

        Score { values }
    }
}

impl Add<f32> for Score {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        if self.values.is_empty() {
            return Score::from_f32(other);
        }

        let values = self.values.iter().map(|a| a + other).collect();

        Score { values }
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if self.values.is_empty() {
            return other;
        }

        let values = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a - b)
            .collect();

        Score { values }
    }
}

impl Sub<f32> for Score {
    type Output = Self;

    fn sub(self, other: f32) -> Self {
        if self.values.is_empty() {
            return Score::from_f32(-other);
        }

        let values = self.values.iter().map(|a| a - other).collect();

        Score { values }
    }
}

impl Mul for Score {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        if self.values.is_empty() {
            return other;
        }

        let values = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a * b)
            .collect();

        Score { values }
    }
}

impl Mul<f32> for Score {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        if self.values.is_empty() {
            return Score::from_f32(other);
        }

        let values = self.values.iter().map(|a| a * other).collect();

        Score { values }
    }
}

impl Div for Score {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if self.values.is_empty() {
            return other;
        }

        let values = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a / b)
            .collect();

        Score { values }
    }
}

impl Div<f32> for Score {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        if self.values.is_empty() {
            return Score::from_f32(other);
        }

        let values = self.values.iter().map(|a| a / other).collect();

        Score { values }
    }
}

impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut values = vec![];

        for score in iter {
            for (i, value) in score.values.iter().enumerate() {
                if values.len() <= i {
                    values.push(*value);
                } else {
                    values[i] += value;
                }
            }
        }

        Score {
            values: Arc::from(values),
        }
    }
}

impl<'a> Sum<&'a Score> for Score {
    fn sum<I: Iterator<Item = &'a Score>>(iter: I) -> Self {
        let mut values = vec![];

        for score in iter {
            for (i, value) in score.values.iter().enumerate() {
                if values.len() <= i {
                    values.push(*value);
                } else {
                    values[i] += value;
                }
            }
        }

        Score {
            values: Arc::from(values),
        }
    }
}
