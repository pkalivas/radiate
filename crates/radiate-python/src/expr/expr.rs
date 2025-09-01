use crate::{
    AnyChromosome, AnyGene, AnyValue, DataType, FilterExpr,
    value::{self, NumericSlotMut},
};
use radiate::{Chromosome, Select, random_provider};
use std::ops::Range;

// let mut_logical =
//     genes().every(2).on(mfun::jitter(0.10))
//     .then( genes().index(1).on_prob(0.5, mfun::gaussian(0.0, 0.25)) );

// let cross_logical =
//     pairs().adjacent().on_prob(0.3, cfun::mean());

// // Plan against the population/individual schema
// let schema = ChromosomeSchema::from(&chromosomes);
// let mut_planned   = plan_unary(mut_logical, &schema);
// let cross_planned = plan_pairs(cross_logical, &schema);

// // Compile to your physical Expr
// let mut expr = Expr::Seq(vec![
//     compile_to_expr(&mut_planned),
//     compile_pairs_to_expr(&cross_planned),
// ]);

// // Run with your existing runtime
// let changed = expr.apply2(&mut chromosomes);

#[derive(Debug, Clone)]
pub enum MutateExpr {
    Uniform(Range<f32>),
    Gaussian(f32, f32),
    Jitter(f32),
}

#[derive(Debug, Clone)]
pub enum CrossoverExpr {
    OnePoint,
    TwoPoint,
    Swap,
    Mean,
}

#[derive(Debug, Clone)]
pub enum SelectExpr {
    All,
    Random(usize),
    Index(usize),
    Range(Range<usize>),
}

#[derive(Debug, Clone)]
pub enum AlterExpr {
    Mutate(MutateExpr),
    Crossover(CrossoverExpr),
}

#[derive(Debug, Clone)]
pub struct Alteration {
    pub name: String,
    pub expr: Expr,
    pub target: String,
    pub p: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprPath<'a> {
    Root,
    Field(&'a str),
    Index(usize),
}

#[derive(Debug, Clone)]
pub enum Expr {
    // structure/navigation
    This,                       // do nothing
    AtField(String, Box<Expr>), // run inner where path == field
    AtIndex(usize, Box<Expr>),  // run inner at a specific index
    All(Box<Expr>),             // map inner across all children (vectors/structs)

    // combinators
    Seq(Vec<Expr>),             // run in order (pipe)
    Prob(f32, Box<Expr>),       // run inner with probability p
    DType(DataType, Box<Expr>), // run inner only if leaf dtype matches

    // filtering
    Filter(FilterExpr),
    // branching
    If(Box<Expr>, Box<Expr>), // if true, run first, else second
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),

    // selection
    Select(SelectExpr, Box<Expr>),

    // leaf ops
    Mut(MutateExpr),
    Cross(CrossoverExpr), // used by pair eval
}

#[derive(Debug, Clone)]
pub struct PlanExpr {
    selection: Vec<SelectExpr>,
    filtering: Vec<FilterExpr>,
    application: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct PipelineExpr {
    steps: Vec<PlanExpr>,
}

impl PipelineExpr {
    pub fn new() -> Self {
        PipelineExpr { steps: Vec::new() }
    }

    pub fn all(self) -> Self {
        self.select(SelectExpr::All)
    }

    pub fn index(self, index: usize) -> Self {
        self.select(SelectExpr::Index(index))
    }

    pub fn random(self, count: usize) -> Self {
        self.select(SelectExpr::Random(count))
    }

    pub fn range(self, range: Range<usize>) -> Self {
        self.select(SelectExpr::Range(range))
    }

    pub fn prob(self, p: f32) -> Self {
        self.filter(FilterExpr::Prob(p))
    }

    pub fn mutate(self, mut_expr: MutateExpr) -> Self {
        self.apply(Expr::Mut(mut_expr))
    }

    pub fn then(self, step: PipelineExpr) -> Self {
        let mut new_steps = self.steps;
        new_steps.extend(step.steps);
        PipelineExpr { steps: new_steps }
    }

    fn select(mut self, sel: SelectExpr) -> Self {
        if let Some(last) = self.steps.last_mut() {
            last.selection.push(sel);
        } else {
            self.steps.push(PlanExpr {
                selection: vec![sel],
                filtering: Vec::new(),
                application: Vec::new(),
            });
        }
        self
    }

    fn filter(mut self, filter: FilterExpr) -> Self {
        if let Some(last) = self.steps.last_mut() {
            last.filtering.push(filter);
        } else {
            self.steps.push(PlanExpr {
                selection: Vec::new(),
                filtering: vec![filter],
                application: Vec::new(),
            });
        }
        self
    }

    fn apply(mut self, expr: Expr) -> Self {
        if let Some(last) = self.steps.last_mut() {
            last.application.push(expr);
        } else {
            self.steps.push(PlanExpr {
                selection: Vec::new(),
                filtering: Vec::new(),
                application: vec![expr],
            });
        }
        self
    }

    pub fn build(self) -> Expr {
        let mut exprs = Vec::new();

        for step in self.steps {
            // 1) Build application chain first (fold from end)
            let mut current = if step.application.is_empty() {
                Expr::This
            } else {
                let mut apps = step.application.into_iter().rev();
                let mut acc = apps.next().unwrap();
                for app in apps {
                    acc = Expr::Seq(vec![app, acc]);
                }
                acc
            };

            // 2) For each filter (reverse), wrap current
            for filter in step.filtering.into_iter().rev() {
                match filter {
                    FilterExpr::Prob(p) => {
                        current = Expr::Prob(p, Box::new(current));
                    }
                }
            }

            // 3) For each selection (reverse), wrap current
            for sel in step.selection.into_iter().rev() {
                current = Expr::Select(sel, Box::new(current));
            }

            exprs.push(current);
        }

        if exprs.len() == 1 {
            exprs.pop().unwrap()
        } else {
            Expr::Seq(exprs)
        }
    }
}

pub trait Eval<I: ?Sized, O> {
    fn eval(&self, input: &I) -> O;
}

pub trait EvalMut<I: ?Sized, O> {
    fn eval_mut(&mut self, input: &mut I) -> O;
}

impl<I: ?Sized, O> EvalMut<I, O> for dyn Eval<I, O> {
    fn eval_mut(&mut self, input: &mut I) -> O {
        self.eval(input)
    }
}

pub enum ExprValue<'a, N> {
    Single(&'a mut N),
    Sequence(&'a mut [N]),
}

pub trait ExprTarget {
    type Value: ExprTarget;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprPath<'b>, ExprValue<'b, Self::Value>);

    fn dtype(&self) -> DataType;

    #[inline]
    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        None
    }
}

pub trait ExprDispatch {
    fn dispatch(&mut self, expr: &mut Expr) -> usize;
}

impl ExprDispatch for AnyChromosome<'_> {
    fn dispatch(&mut self, expr: &mut Expr) -> usize {
        expr.apply(ExprValue::Sequence(self.genes_mut()))
    }
}

impl Expr {
    pub fn apply<'a, N: ExprTarget>(&mut self, input: ExprValue<'a, N>) -> usize {
        let mut changed = 0;
        // Removed debug print to avoid noisy logs in tight loops
        println!("Applying expr: {:?}", self);
        if let ExprValue::Single(n) = input {
            changed += self.apply_single(ExprValue::Single(n));
        } else if let ExprValue::Sequence(ns) = input {
            changed += self.apply_sequence(ExprValue::Sequence(ns));
        }
        changed
    }

    fn apply_single<'a, N: ExprTarget>(&mut self, input: ExprValue<'a, N>) -> usize {
        let mut changed = 0;
        match input {
            ExprValue::Single(value) => match self {
                Expr::This => {}
                Expr::Select(select, inner) => {
                    match select {
                        SelectExpr::All => {
                            changed += inner.apply(ExprValue::Single(value));
                        }
                        SelectExpr::Index(i) => {
                            if *i == 0 {
                                changed += inner.apply(ExprValue::Single(value));
                            }
                        }
                        _ => {
                            // For other selection kinds on a single, do nothing
                        }
                    }
                }
                Expr::AtField(name, inner) => {
                    value.visit(&mut |path, value| {
                        if matches!(path, ExprPath::Field(n) if n == name.as_str()) {
                            changed += inner.apply(value);
                        }
                    });
                }
                Expr::AtIndex(i, expr) => {
                    value.visit(&mut |path, inner| {
                        if matches!(path, ExprPath::Index(j) if j == *i) {
                            changed += expr.apply(inner);
                        }
                    });
                }
                Expr::All(expr) => {
                    value.visit(&mut |_, value| {
                        expr.apply(value);
                    });
                }
                Expr::Seq(list) => {
                    for n in list.iter_mut() {
                        changed += n.apply(ExprValue::Single(value));
                    }
                }
                Expr::Prob(p, inner) => {
                    if *p <= 0.0 || *p > 1.0 {
                        return 0;
                    } else if random_provider::random::<f32>() < *p {
                        return inner.apply(ExprValue::Single(value));
                    } else {
                        return 0;
                    }
                }
                Expr::Mut(mutation) => {
                    value.visit(&mut |_, inner| match inner {
                        ExprValue::Single(n) => {
                            changed += mutation.eval_mut(n);
                        }
                        ExprValue::Sequence(ns) => {
                            for n in ns.iter_mut() {
                                changed += mutation.eval_mut(n);
                            }
                        }
                    });
                }
                _ => {}
            },
            ExprValue::Sequence(ns) => {
                for n in ns.iter_mut() {
                    changed += self.apply(ExprValue::Single(n));
                }
            }
        }
        changed
    }

    fn apply_sequence(&mut self, input: ExprValue<'_, impl ExprTarget>) -> usize {
        let mut changed = 0;
        if let ExprValue::Sequence(ns) = input {
            match self {
                Expr::This => {}
                Expr::AtField(_, inner) => {
                    for n in ns.iter_mut() {
                        changed += inner.apply(ExprValue::Single(n));
                    }
                }
                Expr::Select(select, inner) => match select {
                    SelectExpr::All => {
                        for n in ns.iter_mut() {
                            changed += inner.apply(ExprValue::Single(n));
                        }
                    }
                    SelectExpr::Index(i) => {
                        if *i < ns.len() {
                            changed += inner.apply(ExprValue::Single(&mut ns[*i]));
                        }
                    }
                    SelectExpr::Random(count) => {
                        let indices = random_provider::indexes(0..ns.len());
                        for &i in indices.iter().take(*count) {
                            if i < ns.len() {
                                changed += inner.apply(ExprValue::Single(&mut ns[i]));
                            }
                        }
                    }
                    SelectExpr::Range(range) => {
                        let start = range.start.min(ns.len());
                        let end = range.end.min(ns.len());
                        for n in &mut ns[start..end] {
                            changed += inner.apply(ExprValue::Single(n));
                        }
                    }
                },
                Expr::AtIndex(i, inner) => {
                    changed += inner.apply(ExprValue::Single(&mut ns[*i]));
                }
                Expr::All(inner) => {
                    for n in ns.iter_mut() {
                        changed += inner.apply(ExprValue::Single(n));
                    }
                }
                Expr::Seq(list) => {
                    for n in list.iter_mut() {
                        changed += n.apply(ExprValue::Sequence(ns));
                    }
                }
                Expr::Prob(p, inner) => {
                    if *p <= 0.0 || *p > 1.0 {
                        return 0;
                    } else if random_provider::random::<f32>() < *p {
                        return inner.apply(ExprValue::Sequence(ns));
                    } else {
                        return 0;
                    }
                }
                Expr::Mut(mutation) => {
                    for n in ns.iter_mut() {
                        changed += mutation.eval_mut(n);
                    }
                }
                _ => {}
            }
        }
        changed
    }

    pub fn apply_pair<N: ExprTarget>(&mut self, one: &mut N, two: &mut N) -> usize {
        let mut changed = 0;
        match self {
            Expr::Cross(c) => c.eval_mut(&mut (one, two)),
            Expr::Seq(list) => {
                for e in list.iter_mut() {
                    changed += e.apply_pair(one, two);
                }
                changed
            }
            Expr::Prob(p, inner) => {
                if *p <= 0.0 || *p > 1.0 {
                    return 0;
                } else if random_provider::random::<f32>() < *p {
                    return inner.apply_pair(one, two);
                } else {
                    return 0;
                }
            }
            Expr::DType(dt, inner) => {
                if one.dtype() == *dt && two.dtype() == *dt {
                    inner.apply_pair(one, two)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

impl<'a, N: ExprTarget> EvalMut<(&mut N, &mut N), usize> for CrossoverExpr {
    fn eval_mut(&mut self, input: &mut (&mut N, &mut N)) -> usize {
        match self {
            CrossoverExpr::OnePoint => 0,
            CrossoverExpr::TwoPoint => 0,
            CrossoverExpr::Swap => 0,
            CrossoverExpr::Mean => 0,
        }
    }
}

impl<'a> ExprTarget for AnyChromosome<'a> {
    type Value = AnyGene<'a>;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprPath<'b>, ExprValue<'b, Self::Value>),
    {
        f(ExprPath::Root, ExprValue::Sequence(&mut self.genes_mut()));
    }

    fn dtype(&self) -> DataType {
        DataType::Float16
    }
}

impl<'a> ExprTarget for AnyGene<'a> {
    type Value = AnyValue<'a>;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprPath<'b>, ExprValue<'b, Self::Value>),
    {
        self.allele_mut().visit(f);
    }

    fn dtype(&self) -> DataType {
        DataType::Float16
    }
}

impl<'a> ExprTarget for AnyValue<'a> {
    type Value = Self;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprPath<'b>, ExprValue<'b, Self::Value>),
    {
        match self {
            AnyValue::Struct(pairs) => {
                for (value, field) in pairs.iter_mut() {
                    value.visit(f);
                    f(ExprPath::Field(field.name()), ExprValue::Single(value));
                }
            }
            AnyValue::Vector(vec) => {
                for (i, v) in vec.iter_mut().enumerate() {
                    v.visit(f);
                    f(ExprPath::Index(i), ExprValue::Single(v));
                }
            }
            _ => {
                f(ExprPath::Root, ExprValue::Single(self));
            }
        }
    }

    fn dtype(&self) -> DataType {
        self.dtype()
    }

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        self.numeric_mut()
    }
}

impl ExprTarget for Vec<f32> {
    type Value = f32;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprPath<'b>, ExprValue<'b, Self::Value>),
    {
        f(ExprPath::Root, ExprValue::Sequence(self.as_mut_slice()));
    }

    fn dtype(&self) -> DataType {
        DataType::Float32
    }
}

impl ExprTarget for f32 {
    type Value = Self;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprPath<'b>, ExprValue<'b, Self::Value>),
    {
        f(ExprPath::Root, ExprValue::Single(self));
    }

    fn dtype(&self) -> DataType {
        DataType::Float32
    }

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        Some(NumericSlotMut::F32(self))
    }
}

impl ExprDispatch for Vec<f32> {
    fn dispatch(&mut self, expr: &mut Expr) -> usize {
        expr.apply(ExprValue::Sequence(self.as_mut_slice()))
    }
}

impl Alteration {
    pub fn new(name: String, expr: Expr, target: String, p: f32) -> Self {
        Alteration {
            name,
            expr,
            target,
            p,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let nested_value = AnyValue::Struct(vec![
            (AnyValue::StrOwned("a".into()), "a".into()),
            (AnyValue::Float32(2.0), "target".into()),
            (
                AnyValue::Vector(Box::new(vec![
                    AnyValue::Float32(3.0),
                    AnyValue::Struct(vec![
                        (AnyValue::StrOwned("t".into()), "a".into()),
                        (AnyValue::Float32(4.0), "target".into()),
                    ]),
                ])),
                "list".into(),
            ),
        ]);

        let nested_value2 = AnyValue::Struct(vec![
            (AnyValue::StrOwned("a".into()), "a".into()),
            (AnyValue::Float32(5.0), "target".into()),
            (
                AnyValue::Vector(Box::new(vec![
                    AnyValue::Float32(3.0),
                    AnyValue::Struct(vec![
                        (AnyValue::StrOwned("t".into()), "a".into()),
                        (AnyValue::Float32(1.0), "target".into()),
                    ]),
                ])),
                "list".into(),
            ),
        ]);
        let gene = AnyGene::new(nested_value.clone());
        let gene2 = AnyGene::new(nested_value2.clone());
        let mut chromosomes = AnyChromosome::new(vec![gene, gene2]);

        let mut vec = vec![0.0f32, 1.0, 2.0, 3.0, 4.0];

        let mut expr = PipelineExpr::new()
            .range(1..4)
            .prob(1.0)
            .mutate(MutateExpr::Jitter(0.5))
            .then(
                PipelineExpr::new()
                    .index(0)
                    .prob(0.99)
                    .mutate(MutateExpr::Gaussian(0.0, 0.25)),
            )
            .build();

        vec.dispatch(&mut expr);

        // println!("Nested value after alteration: {:?}", nested_value);
        // Expr::AtIndex(
        //     1,
        //     Box::new(Expr::AtField(
        //         "target".into(),
        //         Box::new(Expr::Mut(MutateExpr::Jitter(0.5))),
        //     )),
        // )
        // .apply2(&mut chromosomes);
        println!("Nested value after alteration: {:?}", vec);
    }
}
