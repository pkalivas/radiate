use crate::{ExprEval, ExprNode, MetricSet, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompareOp {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
}

#[derive(Clone)]
pub struct CompareExprNode<'a> {
    lhs: Box<ExprNode<'a>>,
    rhs: Box<ExprNode<'a>>,
    op: CompareOp,
}

impl<'a> CompareExprNode<'a> {
    pub fn new(lhs: ExprNode<'a>, rhs: ExprNode<'a>, op: CompareOp) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
        }
    }
}

impl ExprEval<MetricSet> for CompareExprNode<'_> {
    fn eval<'a>(&'a mut self, metrics: &'a MetricSet) -> Value<'a> {
        let lhs = self.lhs.eval(metrics).extract::<f32>().unwrap_or_default();
        let rhs = self.rhs.eval(metrics).extract::<f32>().unwrap_or_default();

        let result = match self.op {
            CompareOp::Lt => lhs < rhs,
            CompareOp::Lte => lhs <= rhs,
            CompareOp::Gt => lhs > rhs,
            CompareOp::Gte => lhs >= rhs,
            CompareOp::Eq => (lhs - rhs).abs() <= f32::EPSILON,
        };

        Value::Bool(result)
    }
}
