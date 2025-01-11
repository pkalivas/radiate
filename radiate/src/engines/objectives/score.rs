use std::hash::Hash;

/// A score is a value that can be used to compare the fitness of two individuals and represents
/// the 'fitness' of an individual within the genetic algorithm.
/// The score can be a single value or multiple values, depending on the problem being solved.
/// For ease of use the `Score` struct provides methods
/// to convert the score to a single value, an integer, a string, or a vector of `f32` values.
///
/// Note: The reason it is a Vec is for multi-objective optimization problems. This allows for multiple
/// fitness values to be returned from the fitness function.
///
#[derive(Clone, PartialEq)]
pub struct Score {
    pub values: Vec<f32>,
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
        Score { values }
    }

    pub fn from_f32(value: f32) -> Self {
        if value.is_nan() {
            panic!("Score value cannot be NaN")
        }

        Score {
            values: vec![value],
        }
    }

    pub fn from_int(value: i32) -> Self {
        Score {
            values: vec![value as f32],
        }
    }

    pub fn from_usize(value: usize) -> Self {
        Score {
            values: vec![value as f32],
        }
    }

    pub fn from_string(value: &str) -> Self {
        Score {
            values: vec![value.parse::<f32>().unwrap()],
        }
    }

    pub fn as_f32(&self) -> f32 {
        self.values[0]
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

impl std::fmt::Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.values)
    }
}

impl Hash for Score {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut hash = 0;
        for value in &self.values {
            hash ^= value.to_bits();
        }
        hash.hash(state);
    }
}
