use std::fmt::Debug;

use crate::{AnyChromosome, AnyGene, AnyValue, NumericSlotMut};
use radiate::{Chromosome, Gene};

pub enum ExprValue<'a, T> {
    Single(&'a mut T),
    Sequence(&'a mut [T]),
    Pair(&'a mut T, &'a mut T),
    SequencePair(&'a mut [T], &'a mut [T]),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExprNodeMeta<'a> {
    Name(&'a str),
    Index(usize),
    None,
}

pub trait ExprNode: Debug {
    type Value: ExprNode;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprNodeMeta<'b>, ExprValue<'b, Self::Value>) -> bool;

    fn is_sequence(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn get_by_name(&mut self, _: &str) -> Option<&mut Self::Value> {
        None
    }

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        None
    }

    fn as_mut_slice(&mut self) -> Option<&mut [Self::Value]> {
        None
    }
}

impl<'a> ExprNode for AnyChromosome<'a> {
    type Value = AnyGene<'a>;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprNodeMeta<'b>, ExprValue<'b, Self::Value>) -> bool,
    {
        f(
            ExprNodeMeta::None,
            ExprValue::Sequence(&mut self.genes_mut()),
        );
    }
}

impl<'a> ExprNode for AnyGene<'a> {
    type Value = AnyValue<'a>;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprNodeMeta<'b>, ExprValue<'b, Self::Value>) -> bool,
    {
        self.allele_mut().visit(f);
    }

    fn get_by_name(&mut self, name: &str) -> Option<&mut Self::Value> {
        self.allele_mut().get_by_name(name)
    }

    fn is_leaf(&self) -> bool {
        self.allele().is_leaf()
    }

    fn is_sequence(&self) -> bool {
        self.allele().is_sequence()
    }

    fn as_mut_slice(&mut self) -> Option<&mut [Self::Value]> {
        self.allele_mut().as_mut_slice()
    }
}

impl<'a> ExprNode for AnyValue<'a> {
    type Value = Self;

    fn visit<F>(&mut self, f: &mut F)
    where
        F: for<'b> FnMut(ExprNodeMeta<'b>, ExprValue<'b, Self::Value>) -> bool,
    {
        match self {
            AnyValue::Struct(pairs) => {
                for (value, meta) in pairs.iter_mut() {
                    let expr_value = if value.is_sequence() {
                        if let Some(slice) = value.as_mut_slice() {
                            ExprValue::Sequence(slice)
                        } else {
                            ExprValue::Single(value)
                        }
                    } else {
                        ExprValue::Single(value)
                    };

                    if !f(ExprNodeMeta::Name(meta.name()), expr_value) {
                        value.visit(f);
                    }
                }
            }
            AnyValue::Vector(vec) => {
                f(ExprNodeMeta::None, ExprValue::Sequence(vec.as_mut_slice()));
            }
            _ => {
                f(ExprNodeMeta::None, ExprValue::Single(self));
            }
        }
    }

    fn get_by_name(&mut self, name: &str) -> Option<&mut Self::Value> {
        self.get_nested_value(name)
    }

    fn numeric_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        self.numeric_mut()
    }

    fn is_leaf(&self) -> bool {
        !self.is_nested()
    }

    fn is_sequence(&self) -> bool {
        matches!(self, AnyValue::Vector(_))
    }

    fn as_mut_slice(&mut self) -> Option<&mut [Self::Value]> {
        self.as_mut_slice()
    }
}
