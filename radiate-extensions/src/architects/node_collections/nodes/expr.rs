use std::ops::Range;

pub enum Expr<T> {
    Val(T),
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Sum,
    Prod,
    Neg,
    Sqrt,
    Abs,
    Exp,
    Log,
    Sin,
    Cos,
    Tan,
    Floor,
    Ceil,
    GT,
    LT,
    Max,
    Min,
    Weight(Range<T>),
    Var(&'static str, usize),
    Const(&'static str, T),
    Fn(&'static str, usize, fn(&[T]) -> T),
    Sigmoid,
    Tanh,
    Relu,
    LeakyRelu,
    Identity,
    Mish,
    Swish,
    MaxOut,
    MaxPool,
    AvgPool,
    Conv2D,
}

pub enum Arity {
    Nullary,
    Unary,
    Binary,
    Ternary,
    Nary,
}

impl<T> Expr<T> {
    pub fn arity(&self) -> Arity {
        match self {
            Expr::Val(_) => Arity::Nullary,
            Expr::Add | Expr::Sub | Expr::Mul | Expr::Div | Expr::Pow => Arity::Binary,
            Expr::Sum | Expr::Prod => Arity::Nary,
            Expr::Neg
            | Expr::Sqrt
            | Expr::Abs
            | Expr::Exp
            | Expr::Log
            | Expr::Sin
            | Expr::Cos
            | Expr::Tan
            | Expr::Floor
            | Expr::Ceil
            | Expr::Sigmoid
            | Expr::Tanh
            | Expr::Relu
            | Expr::LeakyRelu
            | Expr::Identity
            | Expr::Mish
            | Expr::Swish
            | Expr::MaxOut
            | Expr::MaxPool
            | Expr::AvgPool
            | Expr::Conv2D => Arity::Unary,
            Expr::GT | Expr::LT | Expr::Max | Expr::Min => Arity::Nary,
            Expr::Weight(_) => Arity::Unary,
            Expr::Var(_, _) => Arity::Nullary,
            Expr::Const(_, _) => Arity::Nullary,
            Expr::Fn(_, _, _) => Arity::Nary,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Expr::Val(_) => "Val",
            Expr::Add => "Add",
            Expr::Sub => "Sub",
            Expr::Mul => "Mul",
            Expr::Div => "Div",
            Expr::Pow => "Pow",
            Expr::Sum => "Sum",
            Expr::Prod => "Prod",
            Expr::Neg => "Neg",
            Expr::Sqrt => "Sqrt",
            Expr::Abs => "Abs",
            Expr::Exp => "Exp",
            Expr::Log => "Log",
            Expr::Sin => "Sin",
            Expr::Cos => "Cos",
            Expr::Tan => "Tan",
            Expr::Floor => "Floor",
            Expr::Ceil => "Ceil",
            Expr::GT => "GT",
            Expr::LT => "LT",
            Expr::Max => "Max",
            Expr::Min => "Min",
            Expr::Weight(_) => "Weight",
            Expr::Var(_, _) => "Var",
            Expr::Const(_, _) => "Const",
            Expr::Fn(_, _, _) => "Fn",
            Expr::Sigmoid => "Sigmoid",
            Expr::Tanh => "Tanh",
            Expr::Relu => "Relu",
            Expr::LeakyRelu => "LeakyRelu",
            Expr::Identity => "Identity",
            Expr::Mish => "Mish",
            Expr::Swish => "Swish",
            Expr::MaxOut => "MaxOut",
            Expr::MaxPool => "MaxPool",
            Expr::AvgPool => "AvgPool",
            Expr::Conv2D => "Conv2D",
        }
    }
}
