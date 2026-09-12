use crate::{Expr, ExprNode};
use radiate_error::{RadiateError, radiate_err};

impl TryFrom<Expr> for f32 {
    type Error = RadiateError;

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        if let ExprNode::Literal(lit) = value.node {
            let extracted = lit.extract::<f32>();
            match extracted {
                Some(val) => Ok(val),
                None => Err(radiate_err!(Expr: "Failed to extract f32 from literal")),
            }
        } else {
            Err(radiate_err!(Expr: "Cannot convert Expr to f32: Expr is not a literal"))
        }
    }
}
