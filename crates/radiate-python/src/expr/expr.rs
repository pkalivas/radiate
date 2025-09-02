use crate::{AnyChromosome, ExprNode, ExprNodeMeta, ExprValue, FilterExpr};
use radiate::{Chromosome, random_provider};
use std::ops::Range;

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
    Name(String),
}

#[derive(Debug, Clone)]
pub struct Alteration {
    pub name: String,
    pub expr: PyExpr,
    pub target: String,
    pub p: f32,
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
pub enum PyExpr {
    // structure/navigation
    This,                        // do nothing
    AtIndex(usize, Box<PyExpr>), // run inner at a specific index
    All(Box<PyExpr>),            // map inner across all children (vectors/structs)

    // combinators
    Seq(Vec<PyExpr>),       // run in order (pipe)
    Prob(f32, Box<PyExpr>), // run inner with probability p

    // filtering
    Filter(FilterExpr),

    // selection
    Select(SelectExpr, Box<PyExpr>),

    // leaf ops
    Mut(MutateExpr),
    Cross(CrossoverExpr),
}

#[derive(Debug, Clone)]
pub struct PlanExpr {
    selection: Vec<SelectExpr>,
    filtering: Vec<FilterExpr>,
    application: Vec<PyExpr>,
}

#[derive(Debug, Clone)]
pub struct PyAlterExpr {
    steps: Vec<PlanExpr>,
}

impl PyAlterExpr {
    pub fn new() -> Self {
        PyAlterExpr { steps: Vec::new() }
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

    pub fn name(self, name: String) -> Self {
        self.select(SelectExpr::Name(name))
    }

    pub fn prob(self, p: f32) -> Self {
        self.filter(FilterExpr::Prob(p))
    }

    pub fn mutate(self, mut_expr: MutateExpr) -> Self {
        self.apply(PyExpr::Mut(mut_expr))
    }

    pub fn cross(self, cross_expr: CrossoverExpr) -> Self {
        self.apply(PyExpr::Cross(cross_expr))
    }

    pub fn then(self, step: PyAlterExpr) -> Self {
        let mut new_steps = self.steps;
        new_steps.extend(step.steps);
        PyAlterExpr { steps: new_steps }
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

    fn apply(mut self, expr: PyExpr) -> Self {
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

    pub fn build(self) -> PyExpr {
        let mut exprs = Vec::new();

        for step in self.steps {
            let mut current = if step.application.is_empty() {
                PyExpr::This
            } else {
                let mut apps = step.application.into_iter().rev();
                let mut acc = apps.next().unwrap();
                for app in apps {
                    acc = PyExpr::Seq(vec![app, acc]);
                }
                acc
            };

            for filter in step.filtering.into_iter().rev() {
                match filter {
                    FilterExpr::Prob(p) => {
                        current = PyExpr::Prob(p, Box::new(current));
                    }
                }
            }

            for sel in step.selection.into_iter().rev() {
                current = PyExpr::Select(sel, Box::new(current));
            }

            exprs.push(current);
        }

        if exprs.len() == 1 {
            exprs.pop().unwrap()
        } else {
            PyExpr::Seq(exprs)
        }
    }
}

pub trait ExprDispatch {
    fn dispatch(&mut self, expr: &PyExpr) -> usize;
}

impl ExprDispatch for AnyChromosome<'_> {
    fn dispatch(&mut self, expr: &PyExpr) -> usize {
        expr.apply(ExprValue::Sequence(self.genes_mut()))
    }
}

impl<'a> ExprDispatch for (&mut AnyChromosome<'a>, &mut AnyChromosome<'a>) {
    fn dispatch(&mut self, expr: &PyExpr) -> usize {
        expr.apply(ExprValue::SequencePair(
            self.0.genes_mut(),
            self.1.genes_mut(),
        ))
    }
}

impl PyExpr {
    pub fn apply<'a, N: ExprNode>(&self, input: ExprValue<'a, N>) -> usize {
        let mut changed = 0;

        match self {
            PyExpr::Prob(p, inner) => {
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
            PyExpr::Select(select, inner) => match select {
                SelectExpr::Name(name) => {
                    input.visit(&mut |meta, value| {
                        if let ExprNodeMeta::Name(n) = meta {
                            if n == name {
                                changed += inner.apply(value);
                                return true;
                            }
                        }
                        false
                    });
                }
                _ => {}
            },
            PyExpr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::Single(input));
                }
            }
            PyExpr::Mut(mutation) => {
                input.visit(&mut |_, inner| match inner {
                    ExprValue::Single(n) => {
                        changed += mutation.apply_mutator(n);
                        true
                    }
                    ExprValue::Sequence(ns) => {
                        for n in ns.iter_mut() {
                            changed += mutation.apply_mutator(n);
                        }
                        true
                    }
                    _ => false,
                });
            }
            _ => {}
        }
        changed
    }

    fn apply_sequence<'a, N: ExprNode>(&self, input: &mut [N]) -> usize {
        let mut changed = 0;

        match self {
            PyExpr::Select(select, inner) => match select {
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
                SelectExpr::Name(name) => {
                    for n in input.iter_mut() {
                        n.visit(&mut |meta, value| {
                            if let ExprNodeMeta::Name(n) = meta {
                                if n == name {
                                    changed += inner.apply(value);
                                    return true;
                                }
                            }

                            false
                        });
                    }
                }
            },
            PyExpr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::Sequence(input));
                }
            }
            PyExpr::Mut(mutation) => {
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
            PyExpr::Select(select, inner) => match select {
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
            PyExpr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::Pair(one, two));
                }
            }
            PyExpr::Mut(mutation) => {
                changed += mutation.apply_mutator(one);
                changed += mutation.apply_mutator(two);
            }
            PyExpr::Cross(crossover) => {
                changed += crossover.apply_crossover(ExprValue::Pair(one, two));
            }
            _ => {}
        }

        changed
    }

    fn apply_sequence_pair<N: ExprNode>(&self, one: &mut [N], two: &mut [N]) -> usize {
        let mut changed = 0;
        match self {
            PyExpr::Select(select, inner) => match select {
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
                SelectExpr::Name(name) => {
                    for (one_val, two_val) in one.iter_mut().zip(two.iter_mut()) {
                        let one_by_name = one_val.get_by_name(name);
                        let two_by_name = two_val.get_by_name(name);

                        if let (Some(one), Some(two)) = (one_by_name, two_by_name) {
                            if let (Some(one_slice), Some(two_slice)) =
                                (one.as_mut_slice(), two.as_mut_slice())
                            {
                                changed +=
                                    inner.apply(ExprValue::SequencePair(one_slice, two_slice));
                            } else {
                                changed += inner.apply(ExprValue::Pair(one, two));
                            }
                        }
                    }
                }
            },
            PyExpr::Seq(list) => {
                for n in list.iter() {
                    changed += n.apply(ExprValue::SequencePair(one, two));
                }
            }
            PyExpr::Mut(mutation) => {
                for n in one.iter_mut() {
                    changed += mutation.apply_mutator(n);
                }
                for n in two.iter_mut() {
                    changed += mutation.apply_mutator(n);
                }
            }
            PyExpr::Cross(crossover) => {
                changed += crossover.apply_crossover(ExprValue::SequencePair(one, two));
            }
            _ => {}
        }

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnyGene, AnyValue};

    #[test]
    fn it_works() {
        let nested_value = AnyValue::Struct(vec![
            (AnyValue::Float64(3.0), "nuumber".into()),
            (AnyValue::StrOwned("hello".into()), "text".into()),
            (AnyValue::Bool(true), "flag".into()),
            (
                AnyValue::Struct(vec![
                    (AnyValue::StrOwned("key".into()), "value".into()),
                    (
                        AnyValue::Vector(Box::new(vec![
                            AnyValue::Int64(0),
                            AnyValue::Int64(1),
                            AnyValue::Int64(2),
                        ])),
                        "list".into(),
                    ),
                ]),
                "complex".into(),
            ),
        ]);

        let nested_value2 = AnyValue::Struct(vec![
            (AnyValue::Float64(3.0), "nuumber".into()),
            (AnyValue::StrOwned("hello".into()), "text".into()),
            (AnyValue::Bool(true), "flag".into()),
            (
                AnyValue::Struct(vec![
                    (AnyValue::StrOwned("key".into()), "value".into()),
                    (
                        AnyValue::Vector(Box::new(vec![
                            AnyValue::Int64(4),
                            AnyValue::Int64(5),
                            AnyValue::Int64(6),
                        ])),
                        "list".into(),
                    ),
                ]),
                "complex".into(),
            ),
        ]);

        let gene = AnyGene::new(nested_value.clone());
        let gene2 = AnyGene::new(nested_value2.clone());
        let mut chromosomes = AnyChromosome::new(vec![gene, gene2]);

        let expr = PyAlterExpr::new()
            .name("list".into())
            .prob(0.99)
            .mutate(MutateExpr::Uniform(-10.0..10.0))
            .then(
                PyAlterExpr::new()
                    .index(0)
                    .prob(0.99)
                    .mutate(MutateExpr::Gaussian(0.0, 0.25)),
            )
            .build();

        chromosomes.dispatch(&expr);

        println!("Nested value after alteration: {:#?}", chromosomes);
    }
}
