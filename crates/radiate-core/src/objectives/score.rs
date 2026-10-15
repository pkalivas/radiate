use crate::error::RadiateResult;
use radiate_error::{RadiateError, radiate_bail};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::ops::Index;
use std::sync::Arc;
use std::{borrow::Borrow, fmt::Debug};

/// A trait for types that can be scored.
/// We can use this for a common interface for all scored types.
pub trait Scored {
    fn score(&self) -> Option<&Score>;
}

/// A [Score] is a value that can be used to compare the fitness of two individuals and represents
/// the 'fitness' of an individual within the genetic algorithm.
/// The [Score] can be a single value or multiple values, depending on the problem being solved.
#[derive(Clone, PartialEq)]
pub enum Score {
    Single(f32),
    Multi(Arc<[f32]>),
}

impl Score {
    pub fn from_vec(values: Vec<f32>) -> Self {
        if values.len() == 1 {
            Score::Single(values[0])
        } else {
            Score::Multi(Arc::from(values))
        }
    }

    pub fn is_multi_objective(&self) -> bool {
        match &self {
            Score::Single(_) => false,
            Score::Multi(v) => v.len() > 1,
        }
    }

    pub fn is_single_objective(&self) -> bool {
        match &self {
            Score::Single(_) => true,
            Score::Multi(v) => v.len() == 1,
        }
    }

    pub fn dims(&self) -> usize {
        match &self {
            Score::Single(_) => 1,
            Score::Multi(v) => v.len(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> + '_ {
        match &self {
            Score::Single(v) => std::slice::from_ref(v).iter(),
            Score::Multi(v) => v.iter(),
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Score::Single(_) => 1,
            Score::Multi(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self {
            Score::Single(_) => false,
            Score::Multi(v) => v.is_empty(),
        }
    }

    #[inline]
    pub fn first(&self) -> Option<f32> {
        match &self {
            Score::Single(v) => Some(*v),
            Score::Multi(v) => v.first().cloned(),
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        match &self {
            Score::Single(v) => std::slice::from_ref(v),
            Score::Multi(v) => v,
        }
    }

    pub fn as_f32(&self) -> f32 {
        match &self {
            Score::Single(v) => *v,
            Score::Multi(v) => v.first().cloned().unwrap_or(f32::NAN),
        }
    }

    pub fn as_f64(&self) -> f64 {
        self.first().unwrap_or(f32::NAN) as f64
    }

    pub fn as_i32(&self) -> i32 {
        match &self {
            Score::Single(v) => *v as i32,
            Score::Multi(v) => v[0] as i32,
        }
    }

    pub fn as_usize(&self) -> usize {
        match &self {
            Score::Single(v) => *v as usize,
            Score::Multi(v) => v[0] as usize,
        }
    }

    pub fn scale<F>(&self, func: F) -> Score
    where
        F: Fn(f32) -> f32,
    {
        match &self {
            Score::Single(v) => Score::from(func(*v)),
            Score::Multi(v) => Score::from(v.iter().map(|x| func(*x)).collect::<Vec<f32>>()),
        }
    }

    pub fn merge<F>(&self, other: &Score, func: F) -> RadiateResult<Score>
    where
        F: Fn(f32, f32) -> f32,
    {
        match (&self, &other) {
            (Score::Single(a), Score::Single(b)) => Ok(Score::from(func(*a, *b))),
            (Score::Multi(a), Score::Multi(b)) => {
                if a.len() != b.len() {
                    radiate_bail!(Fitness: "Mismatched lengths when combining scores: {} vs {}", a.len(), b.len());
                }

                Ok(Score::from(
                    a.iter()
                        .zip(b.iter())
                        .map(|(x, y)| func(*x, *y))
                        .collect::<Vec<f32>>(),
                ))
            }
            _ => {
                radiate_bail!(Fitness:
                     "Mismatched score types when combining scores: {:?} vs {:?}", &self, &other)
            }
        }
    }
}

impl Default for Score {
    fn default() -> Self {
        Score::Multi(Vec::new().into())
    }
}

impl AsRef<[f32]> for Score {
    fn as_ref(&self) -> &[f32] {
        match &self {
            Score::Single(v) => std::slice::from_ref(v),
            Score::Multi(v) => v,
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (&self, &other) {
            (Score::Single(a), Score::Single(b)) => a.partial_cmp(b),
            (Score::Single(a), Score::Multi(b)) => a.partial_cmp(&b[0]),
            (Score::Multi(a), Score::Single(b)) => a[0].partial_cmp(b),
            (Score::Multi(a), Score::Multi(b)) => a[0].partial_cmp(&b[0]),
        }
    }
}

impl Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Score::Single(v) => write!(f, "{:?}", v),
            Score::Multi(v) => write!(f, "{:?}", v),
        }
    }
}

impl Eq for Score {}

impl Hash for Score {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self {
            Score::Single(v) => {
                if *v == 0.0 {
                    0_u32.hash(state);
                } else {
                    v.to_bits().hash(state);
                }
            }
            Score::Multi(v) => {
                v.len().hash(state);
                for value in v.iter() {
                    if *value == 0.0 {
                        0_u32.hash(state);
                    } else {
                        value.to_bits().hash(state);
                    }
                }
            }
        }
    }
}

impl Index<usize> for Score {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        match &self {
            Score::Single(v) => v,
            Score::Multi(v) => &v[index],
        }
    }
}

impl<I, S, F> TryFrom<(I, F)> for Score
where
    I: Iterator<Item = S>,
    S: Borrow<Score>,
    F: Fn(f32, f32) -> f32,
{
    type Error = RadiateError;

    fn try_from((scores, func): (I, F)) -> Result<Self, Self::Error> {
        let mut peekable = scores.peekable();
        if peekable.peek().is_none() {
            radiate_bail!(Fitness: "Cannot aggregate an empty list of scores");
        }

        let dims = peekable.peek().map_or_default(|val| val.borrow().dims());

        if dims == 0 {
            radiate_bail!(Fitness: "Cannot aggregate Vec<Score> with zero dimensions");
        } else if dims == 1 {
            let sum = peekable
                .fold(0.0, |acc, score| match &score.borrow() {
                    Score::Single(v) => func(acc, *v),
                    Score::Multi(v) => func(acc, v[0]),
                });
            Ok(Score::from(sum))
        } else {
            let sum = peekable
                .fold(vec![0.0; dims], |mut acc, score| {
                    match &score.borrow() {
                        Score::Single(v) => acc[0] = func(acc[0], *v),
                        Score::Multi(v) => {
                            for (i, value) in v.iter().enumerate() {
                                acc[i] = func(acc[i], *value);
                            }
                        }
                    }
                    acc
                });

            Ok(Score::Multi(sum.into()))
        }
    }
}

impl TryFrom<i16> for Score {
    type Error = RadiateError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Score::Single(value as f32))
    }
}

impl From<Score> for Vec<f32> {
    fn from(score: Score) -> Vec<f32> {
        match score {
            Score::Single(v) => vec![v],
            Score::Multi(v) => v.to_vec(),
        }
    }
}

impl From<f32> for Score {
    fn from(value: f32) -> Self {
        Score::Single(value)
    }
}

impl From<f64> for Score {
    fn from(value: f64) -> Self {
        Score::Single(value as f32)
    }
}

impl From<i32> for Score {
    fn from(value: i32) -> Self {
        Score::Single(value as f32)
    }
}

impl From<i64> for Score {
    fn from(value: i64) -> Self {
        Score::Single(value as f32)
    }
}

impl From<usize> for Score {
    fn from(value: usize) -> Self {
        Score::Single(value as f32)
    }
}

impl From<Vec<f32>> for Score {
    fn from(value: Vec<f32>) -> Self {
        Score::from_vec(value)
    }
}

impl From<Vec<f64>> for Score {
    fn from(value: Vec<f64>) -> Self {
        Score::from_vec(value.into_iter().map(|v| v as f32).collect())
    }
}

impl From<&[f32]> for Score {
    fn from(value: &[f32]) -> Self {
        Score::from_vec(value.to_vec())
    }
}

impl From<&[f64]> for Score {
    fn from(value: &[f64]) -> Self {
        Score::from_vec(value.iter().map(|&v| v as f32).collect())
    }
}

impl From<Vec<i32>> for Score {
    fn from(value: Vec<i32>) -> Self {
        Score::from_vec(value.into_iter().map(|v| v as f32).collect())
    }
}

impl From<Vec<i64>> for Score {
    fn from(value: Vec<i64>) -> Self {
        Score::from_vec(value.into_iter().map(|v| v as f32).collect())
    }
}

impl From<Vec<usize>> for Score {
    fn from(value: Vec<usize>) -> Self {
        Score::from_vec(value.into_iter().map(|v| v as f32).collect())
    }
}

#[cfg(feature = "serde")]
impl Serialize for Score {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_slice().serialize(serializer)
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

        Ok(match vec.len() {
            0 => Score::Single(0.0),
            1 => Score::Single(vec[0]),
            _ => Score::Multi(vec.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_from_vec() {
        let score = Score::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(score.len(), 3);
    }

    #[test]
    fn test_score_from_usize() {
        let score = Score::from(3);
        assert_eq!(score.len(), 1);
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
    fn test_score_add() {
        let score1 = Score::from(vec![1.0, 2.0, 3.0]);
        let score2 = Score::from(vec![4.0, 5.0, 6.0]);
        let score3 = score1.merge(&score2, |a, b| a + b).unwrap();

        assert_eq!(score3.len(), 3);
        assert_eq!(score3.as_f32(), 5.0);
        assert_eq!(score3[0], 5.0);
        assert_eq!(score3[1], 7.0);
        assert_eq!(score3[2], 9.0);
    }

    #[test]
    fn test_score_sub() {
        let score1 = Score::from(vec![5.0, 7.0, 9.0]);
        let score2 = Score::from(vec![4.0, 5.0, 6.0]);
        let score3 = score1.merge(&score2, |a, b| a - b).unwrap();
        assert_eq!(score3.len(), 3);
        assert_eq!(score3.as_f32(), 1.0);
        assert_eq!(score3[0], 1.0);
        assert_eq!(score3[1], 2.0);
        assert_eq!(score3[2], 3.0);
    }

    #[test]
    fn test_score_mul() {
        let score1 = Score::from(vec![1.0, 2.0, 3.0]);
        let score2 = Score::from(vec![4.0, 5.0, 6.0]);
        let score3 = score1.merge(&score2, |a, b| a * b).unwrap();
        assert_eq!(score3.len(), 3);
        assert_eq!(score3.as_f32(), 4.0);
        assert_eq!(score3[0], 4.0);
        assert_eq!(score3[1], 10.0);
        assert_eq!(score3[2], 18.0);
    }

    #[test]
    fn test_score_div() {
        let score1 = Score::from(vec![4.0, 8.0, 12.0]);
        let score2 = Score::from(vec![2.0, 4.0, 6.0]);
        let score3 = score1.merge(&score2, |a, b| a / b).unwrap();
        assert_eq!(score3.len(), 3);
        assert_eq!(score3.as_f32(), 2.0);
        assert_eq!(score3[0], 2.0);
        assert_eq!(score3[1], 2.0);
        assert_eq!(score3[2], 2.0);
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
