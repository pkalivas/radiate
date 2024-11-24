use crate::operations::op::Ops;

use super::node::Node;

pub struct Tracer<T>
where
    T: Clone,
{
    pub input_size: usize,
    pub pending_idx: usize,
    pub args: Vec<T>,
    pub result: Option<T>,
    pub previous_result: Option<T>,
}

impl<T> Tracer<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(input_size: usize) -> Self {
        Tracer {
            input_size,
            pending_idx: 0,
            args: Vec::with_capacity(input_size),
            result: None,
            previous_result: None,
        }
    }

    pub fn add_input(&mut self, value: T) {
        if self.pending_idx == self.input_size {
            panic!("Tracer is not ready to accept more inputs.");
        }

        self.args.push(value);
        self.pending_idx += 1;
    }

    #[inline]
    pub fn eval(&mut self, node: &Node<T>) {
        if self.pending_idx != self.input_size {
            panic!("Tracer is not ready to be evaluated.");
        }

        if !node.enabled {
            self.result = Some(T::default());
        }

        self.previous_result = self.result.clone();
        self.result = match &node.value {
            Ops::Value(ref value) => Some(value.clone()),
            Ops::Const(_, ref value) => Some(value.clone()),
            Ops::Fn(_, _, ref fn_ptr) => Some(fn_ptr(&self.args)),
            Ops::MutableConst(_, _, ref val, _, fn_ptr) => Some(fn_ptr(&self.args, val)),
            Ops::Var(_, _) => Some(self.args[0].clone()),
        };

        self.pending_idx = 0;
        self.args.clear();
    }
}

impl<T> std::fmt::Debug for Tracer<T>
where
    T: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tracer: {{ in_size: {:?}, pending_idx: {:?},  args: {:?}, result: {:?} }}",
            self.input_size, self.pending_idx, self.args, self.result
        )
    }
}
