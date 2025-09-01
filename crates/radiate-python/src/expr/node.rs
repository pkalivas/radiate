use std::fmt::Debug;

use crate::{AnyChromosome, AnyGene, AnyValue};
use radiate::{Chromosome, FloatChromosome, FloatGene, Gene, chromosomes::gene::NumericSlotMut};

pub enum ExprValue<'a, T> {
    Single(&'a mut T),
    Sequence(&'a mut [T]),
    Pair(&'a mut T, &'a mut T),
    SequencePair(&'a mut [T], &'a mut [T]),
}

pub trait ExprNode: Debug {
    type Value: ExprNode;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>);

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        None
    }
}

impl ExprNode for FloatChromosome {
    type Value = FloatGene;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        f(ExprValue::Sequence(&mut self.genes_mut()));
    }
}

impl ExprNode for FloatGene {
    type Value = f32;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        f(ExprValue::Single(&mut self.allele_mut()));
    }
}

impl<'a> ExprNode for AnyChromosome<'a> {
    type Value = AnyGene<'a>;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        f(ExprValue::Sequence(&mut self.genes_mut()));
    }
}

impl<'a> ExprNode for AnyGene<'a> {
    type Value = AnyValue<'a>;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        self.allele_mut().visit(f);
    }
}

impl<'a> ExprNode for AnyValue<'a> {
    type Value = Self;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        match self {
            AnyValue::Struct(pairs) => {
                for (value, _) in pairs.iter_mut() {
                    value.visit(f);
                    // f(ExprValue::Single(value));
                }
            }
            AnyValue::Vector(vec) => {
                for (_, v) in vec.iter_mut().enumerate() {
                    v.visit(f);
                    // f(ExprValue::Single(v));
                }
            }
            _ => {
                f(ExprValue::Single(self));
            }
        }
    }

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        self.numeric_mut()
    }
}

impl ExprNode for Vec<f32> {
    type Value = f32;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        f(ExprValue::Sequence(self.as_mut_slice()));
    }
}

impl ExprNode for f32 {
    type Value = Self;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprValue<'b, Self::Value>),
    {
        f(ExprValue::Single(self));
    }

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        Some(NumericSlotMut::F32(self))
    }
}
