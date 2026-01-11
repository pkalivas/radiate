use pyo3::{pyclass, pymethods};
use radiate::{Rate, rate::CycleShape};
use std::fmt::Debug;

#[pyclass]
#[derive(Clone)]
pub struct PyRate {
    pub rate: Rate,
}

#[pymethods]
impl PyRate {
    pub fn value(&self, index: usize) -> f32 {
        self.rate.value(index)
    }

    #[staticmethod]
    pub fn fixed(value: f32) -> Self {
        PyRate {
            rate: Rate::Fixed(value),
        }
    }

    #[staticmethod]
    pub fn linear(start: f32, end: f32, duration: usize) -> Self {
        PyRate {
            rate: Rate::Linear(start, end, duration),
        }
    }

    #[staticmethod]
    pub fn exponential(start: f32, end: f32, duration: usize) -> Self {
        PyRate {
            rate: Rate::Exponential(start, end, duration),
        }
    }

    #[staticmethod]
    pub fn cyclical(min: f32, max: f32, period: usize, cycle_type: String) -> Self {
        let cycle_shape = match cycle_type.as_str() {
            "sine" => CycleShape::Sine,
            "triangular" => CycleShape::Triangle,
            _ => CycleShape::Sine,
        };

        PyRate {
            rate: Rate::Cyclical(min, max, period, cycle_shape),
        }
    }

    #[staticmethod]
    pub fn stepwise(steps: Vec<(usize, f32)>) -> Self {
        PyRate {
            rate: Rate::Stepwise(steps),
        }
    }
}

impl Debug for PyRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PyRate {{ rate: {:?} }}", self.rate)
    }
}
