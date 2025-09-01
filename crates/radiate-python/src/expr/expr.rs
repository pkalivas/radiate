use crate::{AnyChromosome, ExprNode, ExprValue, FilterExpr};
use radiate::{ArithmeticGene, Chromosome, FloatChromosome, Gene, expr::DataType, random_provider};
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
    Random,
    Index(usize),
    Range(Range<usize>),
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
pub enum Literal {
    Float(f32),
    Double(f64),
    Int(i32),
    Long(i64),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    // structure/navigation
    This,                       // do nothing
    AtField(String, Box<Expr>), // run inner where path == field
    AtIndex(usize, Box<Expr>),  // run inner at a specific index
    All(Box<Expr>),             // map inner across all children (vectors/structs)

    // combinators
    Seq(Vec<Expr>),       // run in order (pipe)
    Prob(f32, Box<Expr>), // run inner with probability p

    // filtering
    Filter(FilterExpr),

    // branching
    If(Box<Expr>, Box<Expr>), // if true, run first, else second
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),

    // selection
    Select(SelectExpr, Box<Expr>),

    // leaf ops
    Mut(MutateExpr),
    Cross(CrossoverExpr),
}

#[derive(Debug, Clone)]
pub struct PlanExpr {
    selection: Vec<SelectExpr>,
    filtering: Vec<FilterExpr>,
    application: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct AlterExpr {
    steps: Vec<PlanExpr>,
}

impl AlterExpr {
    pub fn new() -> Self {
        AlterExpr { steps: Vec::new() }
    }

    pub fn all(self) -> Self {
        self.select(SelectExpr::All)
    }

    pub fn index(self, index: usize) -> Self {
        self.select(SelectExpr::Index(index))
    }

    pub fn random(self) -> Self {
        self.select(SelectExpr::Random)
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

    pub fn cross(self, cross_expr: CrossoverExpr) -> Self {
        self.apply(Expr::Cross(cross_expr))
    }

    pub fn then(self, step: AlterExpr) -> Self {
        let mut new_steps = self.steps;
        new_steps.extend(step.steps);
        AlterExpr { steps: new_steps }
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

            for filter in step.filtering.into_iter().rev() {
                match filter {
                    FilterExpr::Prob(p) => {
                        current = Expr::Prob(p, Box::new(current));
                    }
                }
            }

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

pub trait ExprDispatch {
    fn dispatch(&mut self, expr: &mut Expr) -> usize;
}

impl ExprDispatch for AnyChromosome<'_> {
    fn dispatch(&mut self, expr: &mut Expr) -> usize {
        expr.apply(ExprValue::Sequence(self.genes_mut()))
    }
}

impl Expr {
    pub fn apply<'a, N: ExprNode>(&self, input: ExprValue<'a, N>) -> usize {
        let mut changed = 0;
        println!("Applying expr: {:?}", self);
        match self {
            Expr::Prob(p, inner) => {
                changed += if random_provider::random::<f32>() < *p && (0.0..=1.0).contains(p) {
                    inner.apply(input)
                } else {
                    0
                };
            }
            _ => {
                if let ExprValue::Single(n) = input {
                    changed += self.apply_single(n);
                } else if let ExprValue::Sequence(ns) = input {
                    changed += self.apply_sequence(ns);
                } else if let ExprValue::Pair(a, b) = input {
                    changed += self.apply_pair(a, b);
                } else if let ExprValue::SequencePair(a, b) = input {
                    changed += self.apply_sequence_pair(a, b);
                }
            }
        }

        changed
    }

    fn apply_single<'a, N: ExprNode>(&self, input: &'a mut N) -> usize {
        let mut changed = 0;
        match self {
            Expr::All(expr) => {
                input.visit(&mut |value| {
                    expr.apply(value);
                });
            }
            Expr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::Single(input));
                }
            }
            Expr::Mut(mutation) => {
                input.visit(&mut |inner| match inner {
                    ExprValue::Single(n) => {
                        changed += mutation.apply_mutator(n);
                    }
                    ExprValue::Sequence(ns) => {
                        for n in ns.iter_mut() {
                            changed += mutation.apply_mutator(n);
                        }
                    }
                    _ => {}
                });
            }
            _ => {}
        }
        changed
    }

    fn apply_sequence<'a, N: ExprNode>(&self, input: &mut [N]) -> usize {
        let mut changed = 0;
        match self {
            Expr::AtField(_, inner) => {
                for n in input.iter_mut() {
                    changed += inner.apply(ExprValue::Single(n));
                }
            }
            Expr::Select(select, inner) => match select {
                SelectExpr::All => {
                    for n in input.iter_mut() {
                        changed += inner.apply(ExprValue::Single(n));
                    }
                }
                SelectExpr::Index(i) => {
                    if *i < input.len() {
                        changed += inner.apply(ExprValue::Single(&mut input[*i]));
                    }
                }
                SelectExpr::Random => {
                    for i in 0..input.len() {
                        if random_provider::bool(0.5) {
                            changed += inner.apply(ExprValue::Single(&mut input[i]));
                        }
                    }
                }
                SelectExpr::Range(range) => {
                    let start = range.start.min(input.len());
                    let end = range.end.min(input.len());
                    for n in &mut input[start..end] {
                        changed += inner.apply(ExprValue::Single(n));
                    }
                }
            },
            Expr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::Sequence(input));
                }
            }
            Expr::Mut(mutation) => {
                for n in input.iter_mut() {
                    changed += mutation.apply_mutator(n);
                }
            }
            _ => {}
        }

        changed
    }

    fn apply_pair<'a, N: ExprNode>(&self, one: &mut N, two: &mut N) -> usize {
        let mut changed = 0;
        match self {
            Expr::Select(select, inner) => match select {
                SelectExpr::All => {
                    inner.apply(ExprValue::Pair(one, two));
                }

                SelectExpr::Random => {
                    if random_provider::bool(0.5) {
                        changed += inner.apply(ExprValue::Pair(one, two));
                    }
                }
                _ => {}
            },
            Expr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::Pair(one, two));
                }
            }
            Expr::Mut(mutation) => {
                changed += mutation.apply_mutator(one);
                changed += mutation.apply_mutator(two);
            }
            Expr::Cross(crossover) => {
                changed += crossover.apply_crossover(ExprValue::Pair(one, two));
            }
            _ => {}
        }

        changed
    }

    fn apply_sequence_pair<N: ExprNode>(&self, one: &mut [N], two: &mut [N]) -> usize {
        let mut changed = 0;
        match self {
            Expr::Select(select, inner) => match select {
                SelectExpr::All => {
                    inner.apply(ExprValue::SequencePair(one, two));
                }
                SelectExpr::Index(i) => {
                    changed += inner.apply(ExprValue::Pair(&mut one[*i], &mut two[*i]));
                }
                SelectExpr::Random => {
                    for i in 0..one.len() {
                        if random_provider::bool(0.5) {
                            changed += inner.apply(ExprValue::Pair(&mut one[i], &mut two[i]));
                        }
                    }
                }
                SelectExpr::Range(range) => {
                    let start = range.start.min(one.len());
                    let end = range.end.min(two.len());
                    for i in start..end {
                        changed += inner.apply(ExprValue::Pair(&mut one[i], &mut two[i]));
                    }
                }
            },
            Expr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::SequencePair(one, two));
                }
            }
            Expr::Mut(mutation) => {
                for n in one.iter_mut() {
                    changed += mutation.apply_mutator(n);
                }
                for n in two.iter_mut() {
                    changed += mutation.apply_mutator(n);
                }
            }
            Expr::Cross(crossover) => {
                changed += crossover.apply_crossover(ExprValue::SequencePair(one, two));
            }
            _ => {}
        }

        changed
    }
}

impl ExprDispatch for Vec<f32> {
    fn dispatch(&mut self, expr: &mut Expr) -> usize {
        expr.apply(ExprValue::Sequence(self.as_mut_slice()))
    }
}

impl ExprDispatch for (&mut Vec<f32>, &mut Vec<f32>) {
    fn dispatch(&mut self, expr: &mut Expr) -> usize {
        expr.apply(ExprValue::SequencePair(
            self.0.as_mut_slice(),
            self.1.as_mut_slice(),
        ))
    }
}

impl ExprDispatch for FloatChromosome {
    fn dispatch(&mut self, expr: &mut Expr) -> usize {
        expr.apply(ExprValue::Sequence(self.genes_mut()))
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

//////
///
/// ///

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnyGene, AnyValue};

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

        let mut vec = vec![0.0f32; 10];
        let mut two = vec![1.0f32; 10];

        let mut expr = AlterExpr::new()
            .all()
            .prob(0.99)
            .cross(CrossoverExpr::OnePoint)
            .then(
                AlterExpr::new()
                    .all()
                    .prob(0.99)
                    .mutate(MutateExpr::Jitter(0.1)),
            )
            .build();

        (&mut vec, &mut two).dispatch(&mut expr);

        let mut expr = AlterExpr::new()
            .range(1..4)
            .prob(1.0)
            .mutate(MutateExpr::Jitter(0.5))
            .then(
                AlterExpr::new()
                    .index(0)
                    .prob(0.99)
                    .mutate(MutateExpr::Gaussian(0.0, 0.25)),
            )
            .build();

        println!("Nested value after alteration: {:#?}", vec);
        println!("Nested value after alteration: {:#?}", two);

        let mut expr = AlterExpr::new()
            .range(1..4)
            .prob(0.99)
            .mutate(MutateExpr::Jitter(0.5))
            .then(
                AlterExpr::new()
                    .index(0)
                    .prob(0.99)
                    .mutate(MutateExpr::Gaussian(0.0, 0.25)),
            )
            .build();

        chromosomes.dispatch(&mut expr);

        println!("Nested value after alteration: {:#?}", chromosomes);
    }
}
