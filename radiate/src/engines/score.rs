use std::hash::Hash;

pub struct Score {
    pub values: Vec<f32>,
}

unsafe impl Send for Score {}

unsafe impl Sync for Score {}

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

    pub fn as_float(&self) -> f32 {
        if self.values.len() > 1 {
            panic!("Score has multiple values, cannot be converted to float")
        }

        self.values[0]
    }

    pub fn as_int(&self) -> i32 {
        if self.values.len() > 1 {
            panic!("Score has multiple values, cannot be converted to int")
        }

        self.values[0] as i32
    }

    pub fn as_string(&self) -> String {
        if self.values.len() > 1 {
            panic!("Score has multiple values, cannot be converted to string")
        }

        self.values[0].to_string()
    }

    pub fn as_usize(&self) -> usize {
        if self.values.len() > 1 {
            panic!("Score has multiple values, cannot be converted to usize")
        }

        self.values[0] as usize
    }
}

impl Clone for Score {
    fn clone(&self) -> Self {
        Score {
            values: self.values.clone(),
        }
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
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

impl Eq for Score {}

impl Hash for Score {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut hash = 0;
        for value in &self.values {
            hash ^= value.to_bits();
        }
        hash.hash(state);
    }
}
