pub enum Arity {
    Nullary,
    Unary,
    Binary,
    Ternary,
    Nary(usize),
    Free,
}

pub struct NodeCell<T> {
    pub value: T,
}
