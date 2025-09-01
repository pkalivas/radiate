use crate::{Expr, ExprNode, ExprValue};

impl Expr {
    pub fn apply_crossover<'a, T: ExprNode>(&mut self, input: ExprValue<'a, T>) -> usize {
        let mut changed = 0;
        // if let ExprValue::Pair(one, two) = input {
        //     changed += self.apply_pair(one, two);
        // }
        changed
    }
}
