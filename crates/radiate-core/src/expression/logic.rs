use crate::{ExprEval, ExprNode, MetricSet, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicOp {
    And,
    Or,
}

#[derive(Clone)]
pub struct LogicExprNode<'a> {
    lhs: Box<ExprNode<'a>>,
    rhs: Box<ExprNode<'a>>,
    op: LogicOp,
}

impl<'a> LogicExprNode<'a> {
    pub fn new(lhs: ExprNode<'a>, rhs: ExprNode<'a>, op: LogicOp) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
        }
    }
}

impl ExprEval<MetricSet> for LogicExprNode<'_> {
    fn eval<'a>(&'a mut self, metrics: &'a MetricSet) -> Value<'a> {
        match self.op {
            LogicOp::And => {
                let lhs = self.lhs.eval(metrics).as_bool();
                if !lhs {
                    return Value::Bool(false);
                }
                Value::Bool(self.rhs.eval(metrics).as_bool())
            }
            LogicOp::Or => {
                let lhs = self.lhs.eval(metrics).as_bool();
                if lhs {
                    return Value::Bool(true);
                }
                Value::Bool(self.rhs.eval(metrics).as_bool())
            }
        }
    }
}
