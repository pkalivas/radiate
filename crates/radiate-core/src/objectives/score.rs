#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
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
#[repr(transparent)]
pub struct Score {
    pub values: Arc<[f32]>,
}

impl Score {
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
        if value.is_nan() {
            panic!("Score value cannot be NaN")
        }

        Score {
            values: Arc::from(vec![value]),
        }
    }
}

impl From<i32> for Score {
    fn from(value: i32) -> Self {
        Score {
            values: Arc::from(vec![value as f32]),
        }
    }
}

impl From<usize> for Score {
    fn from(value: usize) -> Self {
        Score {
            values: Arc::from(vec![value as f32]),
        }
    }
}

impl From<String> for Score {
    fn from(value: String) -> Self {
        Score {
            values: Arc::from(vec![
                value.parse::<f32>().expect("Failed to parse string to f32"),
            ]),
        }
    }
}

impl From<&str> for Score {
    fn from(value: &str) -> Self {
        Score {
            values: Arc::from(vec![
                value.parse::<f32>().expect("Failed to parse string to f32"),
            ]),
        }
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
            return Score::from(other);
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
            return Score::from(-other);
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
            return Score::from(other);
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
            return Score::from(other);
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

#[cfg(feature = "serde")]
impl Serialize for Score {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.values.as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Score {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<f32>::deserialize(deserializer)?;
        for value in &vec {
            if value.is_nan() {
                return Err(serde::de::Error::custom("Score value cannot be NaN"));
            }
        }

        Ok(Score {
            values: Arc::from(vec),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_from_vec() {
        let score = Score::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(score.values.len(), 3);
    }

    #[test]
    fn test_score_from_usize() {
        let score = Score::from(3);
        assert_eq!(score.values.len(), 1);
        assert_eq!(score.as_f32(), 3.0);
        assert_eq!(score.as_i32(), 3);
    }

    #[test]
    fn test_score_from_f32() {
        let score = Score::from(1.0);
        assert_eq!(score.as_f32(), 1.0);
        assert_eq!(score.as_i32(), 1)
    }

    #[test]
    fn test_score_from_i32() {
        let score = Score::from(-5);
        assert_eq!(score.as_f32(), -5.0);
        assert_eq!(score.as_i32(), -5);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_score_can_serialize() {
        let score = Score::from(vec![1.0, 2.0, 3.0]);
        let serialized = serde_json::to_string(&score).expect("Failed to serialize Score");
        let deserialized: Score =
            serde_json::from_str(&serialized).expect("Failed to deserialize Score");
        assert_eq!(score, deserialized);
    }
}
