use crate::Ops;

#[derive(Clone, PartialEq)]
pub struct ValueCell<T> {
    pub name: Option<&'static str>,
    pub min_arity: Option<usize>,
    pub max_arity: Option<usize>,
    pub min_output_arity: Option<usize>,
    pub max_output_arity: Option<usize>,
    pub inner: Ops<T>,
}

impl<T> ValueCell<T> {
    pub fn new(inner: Ops<T>) -> Self {
        Self {
            name: None,
            min_arity: None,
            max_arity: None,
            min_output_arity: None,
            max_output_arity: None,
            inner,
        }
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_min_arity(mut self, min_arity: usize) -> Self {
        self.min_arity = Some(min_arity);
        self
    }

    pub fn with_max_arity(mut self, max_arity: usize) -> Self {
        self.max_arity = Some(max_arity);
        self
    }

    pub fn with_min_output_arity(mut self, min_output_arity: usize) -> Self {
        self.min_output_arity = Some(min_output_arity);
        self
    }

    pub fn with_max_output_arity(mut self, max_output_arity: usize) -> Self {
        self.max_output_arity = Some(max_output_arity);
        self
    }
}
