/// Domain of a variable in a probabilistic graphical model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Domain {
    Discrete(usize),
    Real,
}

/// Observed value type for dataset/evidence
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Discrete(usize), // state index 0..card-1
    Real(f64),       // real value
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Discrete(value)
    }
}

/// A variable in a probabilistic graphical model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: Option<String>,
    pub domain: Domain,
}

impl Variable {
    pub fn new(domain: Domain) -> Self {
        Self { name: None, domain }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl From<Domain> for Variable {
    fn from(domain: Domain) -> Self {
        Variable { name: None, domain }
    }
}

impl From<(String, Domain)> for Variable {
    fn from(domain: (String, Domain)) -> Self {
        Variable {
            name: Some(domain.0),
            domain: domain.1,
        }
    }
}

impl Default for Variable {
    fn default() -> Self {
        Variable {
            name: Some("default".to_string()),
            domain: Domain::Discrete(0),
        }
    }
}
