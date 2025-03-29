use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

pub trait Scored {
    fn values(&self) -> impl AsRef<[f32]>;

    fn as_f32(&self) -> f32 {
        let vals = self.values();
        if vals.as_ref().is_empty() {
            f32::NAN
        } else {
            vals.as_ref()[0]
        }
    }

    fn as_usize(&self) -> usize {
        self.as_f32() as usize
    }

    fn score(&self) -> Option<Score>;
}

impl Scored for Option<Score> {
    fn values(&self) -> impl AsRef<[f32]> {
        match self {
            Some(score) => score.clone(),
            None => Score::default(),
        }
    }

    fn score(&self) -> Option<Score> {
        match self {
            Some(score) => Some(score.clone()),
            None => None,
        }
    }
}

fn guard_nan(value: &f32) {
    if value.is_nan() {
        panic!("Score value cannot be NaN")
    }
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
    pub fn as_f32(&self) -> f32 {
        self.values.get(0).cloned().unwrap_or(f32::NAN)
    }

    pub fn as_i32(&self) -> i32 {
        self.values.get(0).cloned().unwrap_or(f32::NAN) as i32
    }

    pub fn as_string(&self) -> String {
        self.values
            .iter()
            .map(|value| format!("{:.4}", value))
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn as_usize(&self) -> usize {
        self.values.get(0).cloned().unwrap_or(f32::NAN) as usize
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

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, value) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:.4}", value)?;
        }

        Ok(())
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
        let parsed_value = value.parse::<f32>().expect("Failed to parse string to f32");

        Score {
            values: Arc::from(vec![parsed_value]),
        }
    }
}

impl From<&str> for Score {
    fn from(value: &str) -> Self {
        let parsed_value = value.parse::<f32>().expect("Failed to parse string to f32");

        Score {
            values: Arc::from(vec![parsed_value]),
        }
    }
}

impl From<Vec<f32>> for Score {
    fn from(value: Vec<f32>) -> Self {
        for v in &value {
            if v.is_nan() {
                panic!("Score value cannot be NaN")
            }
        }

        Score {
            values: Arc::from(value),
        }
    }
}

impl From<Vec<i32>> for Score {
    fn from(value: Vec<i32>) -> Self {
        let float_values: Vec<f32> = value.iter().map(|&v| v as f32).inspect(guard_nan).collect();

        Score {
            values: Arc::from(float_values),
        }
    }
}

impl From<Vec<usize>> for Score {
    fn from(value: Vec<usize>) -> Self {
        let float_values: Vec<f32> = value.iter().map(|&v| v as f32).inspect(guard_nan).collect();

        Score {
            values: Arc::from(float_values),
        }
    }
}

impl From<Vec<String>> for Score {
    fn from(value: Vec<String>) -> Self {
        let float_values: Vec<f32> = value
            .iter()
            .map(|v| v.parse::<f32>().expect("Failed to parse string to f32"))
            .inspect(guard_nan)
            .collect();

        Score {
            values: Arc::from(float_values),
        }
    }
}

impl From<Vec<&str>> for Score {
    fn from(value: Vec<&str>) -> Self {
        let float_values: Vec<f32> = value
            .iter()
            .map(|&v| v.parse::<f32>().expect("Failed to parse string to f32"))
            .inspect(guard_nan)
            .collect();

        Score {
            values: Arc::from(float_values),
        }
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
