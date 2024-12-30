use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

/// Arity is a way to describe how many inputs an operation expects.
/// It can be zero, a specific number, or any number.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Arity {
    Zero,
    Exact(usize),
    Any,
}

impl From<usize> for Arity {
    fn from(value: usize) -> Self {
        match value {
            0 => Arity::Zero,
            n => Arity::Exact(n),
        }
    }
}

impl Deref for Arity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            Arity::Zero => &0,
            Arity::Exact(n) => n,
            Arity::Any => &0,
        }
    }
}

/// A generic operation type that can represent several kinds of “ops”.
pub enum Operation<T> {
    /// 1) A stateless function operation:
    ///
    /// # Arguments
    ///    - A `&'static str` name (e.g., "Add", "Sigmoid")
    ///    - Arity (how many inputs it takes)
    ///    - Arc<dyn Fn(&[T]) -> T> for the actual function logic
    Fn(&'static str, Arity, Arc<dyn Fn(&[T]) -> T>),
    /// 2) A variable-like operation:
    ///
    /// # Arguments
    ///    - `String` = a name or identifier
    ///    - `usize` = perhaps an index to retrieve from some external context
    Var(&'static str, usize),
    /// 3) A compile-time constant: e.g., 1, 2, 3, etc.
    ///
    /// # Arguments
    ///    - `&'static str` name
    ///    - `T` the actual constant value
    Const(&'static str, T),
    /// 4) A `mutable const` is a constant that can change over time:
    ///
    ///  # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - Current value of type `T`
    /// - An `Arc<dyn Fn() -> T>` for retrieving (or resetting) the value
    /// - An `Arc<dyn Fn(&[T], &T) -> T>` for updating or combining inputs & old value -> new value
    ///
    ///    This suggests a node that can mutate its internal state over time, or
    ///    one that needs a special function to incorporate the inputs into the next state.
    MutableConst {
        name: &'static str,
        arity: Arity,
        value: T,
        get_value: Arc<dyn Fn() -> T>,
        operation: Arc<dyn Fn(&[T], &T) -> T>,
    },
}

/// Base functionality for operations.
impl<T> Operation<T> {
    pub fn name(&self) -> &str {
        match self {
            Operation::Fn(name, _, _) => name,
            Operation::Var(name, _) => name,
            Operation::Const(name, _) => name,
            Operation::MutableConst { name, .. } => name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Operation::Fn(_, arity, _) => *arity,
            Operation::Var(_, _) => Arity::Zero,
            Operation::Const(_, _) => Arity::Zero,
            Operation::MutableConst { arity, .. } => *arity,
        }
    }

    pub fn apply(&self, inputs: &[T]) -> T
    where
        T: Clone,
    {
        match self {
            Operation::Fn(_, _, op) => op(inputs),
            Operation::Var(_, index) => inputs[*index].clone(),
            Operation::Const(_, value) => value.clone(),
            Operation::MutableConst {
                value, operation, ..
            } => operation(inputs, value),
        }
    }

    pub fn new_instance(&self) -> Operation<T>
    where
        T: Clone,
    {
        match self {
            Operation::Fn(name, arity, op) => Operation::Fn(name, *arity, Arc::clone(op)),
            Operation::Var(name, index) => Operation::Var(name, *index),
            Operation::Const(name, value) => Operation::Const(name, value.clone()),
            Operation::MutableConst {
                name,
                arity,
                value: _,
                get_value,
                operation,
            } => Operation::MutableConst {
                name,
                arity: *arity,
                value: (*get_value)(),
                get_value: Arc::clone(get_value),
                operation: Arc::clone(operation),
            },
        }
    }
}

impl<T> Operation<T> {
    pub fn value(value: T) -> Self
    where
        T: Clone + Display,
    {
        let name = Box::leak(Box::new(format!("{}", value)));
        Operation::Const(name, value)
    }

    pub fn constant(name: &'static str, value: T) -> Self {
        Operation::Const(name, value)
    }

    pub fn gt() -> Self
    where
        T: Clone + PartialEq + PartialOrd,
    {
        Operation::Fn(
            ">",
            2.into(),
            Arc::new(|inputs: &[T]| {
                if inputs[0] > inputs[1] {
                    inputs[0].clone()
                } else {
                    inputs[1].clone()
                }
            }),
        )
    }

    pub fn lt() -> Self
    where
        T: Clone + PartialEq + PartialOrd,
    {
        Operation::Fn(
            "<",
            2.into(),
            Arc::new(|inputs: &[T]| {
                if inputs[0] < inputs[1] {
                    inputs[0].clone()
                } else {
                    inputs[1].clone()
                }
            }),
        )
    }

    pub fn identity() -> Self
    where
        T: Clone,
    {
        Operation::Fn(
            "Identity",
            1.into(),
            Arc::new(|inputs: &[T]| inputs[0].clone()),
        )
    }

    pub fn var(index: usize) -> Self
    where
        T: Clone,
    {
        let name = Box::leak(Box::new(format!("var_{}", index)));
        Operation::Var(name, index)
    }
}

unsafe impl Send for Operation<f32> {}
unsafe impl Sync for Operation<f32> {}

impl<T> Clone for Operation<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Operation::Fn(name, arity, op) => Operation::Fn(name, *arity, Arc::clone(op)),
            Operation::Var(name, index) => Operation::Var(name, *index),
            Operation::Const(name, value) => Operation::Const(name, value.clone()),
            Operation::MutableConst {
                name,
                arity,
                value,
                get_value,
                operation,
            } => Operation::MutableConst {
                name,
                arity: *arity,
                value: value.clone(),
                get_value: Arc::clone(get_value),
                operation: Arc::clone(operation),
            },
        }
    }
}

impl<T> PartialEq for Operation<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<T> Display for Operation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<T> Default for Operation<T>
where
    T: Default + Clone,
{
    fn default() -> Self {
        Operation::Const("default", T::default())
    }
}

impl<T> Debug for Operation<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Fn(name, _, _) => write!(f, "Fn: {}", name),
            Operation::Var(name, index) => write!(f, "Var: {}({})", name, index),
            Operation::Const(name, value) => write!(f, "C: {}({:?})", name, value),
            Operation::MutableConst { name, value, .. } => write!(f, "{}({:.2?})", name, value),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use radiate::random_provider;

    #[test]
    fn test_ops() {
        let op = Operation::add();
        assert_eq!(op.name(), "add");
        assert_eq!(op.arity(), Arity::Exact(2));
        assert_eq!(op.apply(&[1_f32, 2_f32]), 3_f32);
        assert_eq!(op.new_instance(), op);
    }

    #[test]
    fn test_random_seed_works() {
        random_provider::set_seed(42);

        let op = Operation::weight();
        let op2 = Operation::weight();

        let o_one = match op {
            Operation::MutableConst { value, .. } => value,
            _ => panic!("Expected MutableConst"),
        };

        let o_two = match op2 {
            Operation::MutableConst { value, .. } => value,
            _ => panic!("Expected MutableConst"),
        };

        println!("o_one: {:?}", o_one);
        println!("o_two: {:?}", o_two);
    }
}
