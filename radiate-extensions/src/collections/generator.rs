use crate::ops::Operation;

pub trait Generator {
    type Input;
    type Output;
    fn generate(&self, input: Self::Input) -> Self::Output;
}

#[derive(Clone, Debug, PartialEq)]
pub struct OperationGenerator<T> {
    operations: Vec<Operation<T>>,
}

impl<T> OperationGenerator<T> {
    pub fn new(operations: Vec<Operation<T>>) -> Self {
        Self { operations }
    }
}
