// use crate::{Evaluate, Expr, ExprResult, ExprSelector, NamedExpr};
// use radiate_error::radiate_err;
// use radiate_utils::SmallStr;

// pub struct ExprRate {
//     inner: NamedExpr,
// }

// impl ExprRate {
//     pub fn new(name: impl Into<SmallStr>, expr: Expr) -> Self {
//         Self {
//             inner: NamedExpr::new(name, expr),
//         }
//     }

//     pub fn inner(&self) -> &NamedExpr {
//         &self.inner
//     }

//     pub fn inner_mut(&mut self) -> &mut NamedExpr {
//         &mut self.inner
//     }
// }

// impl<'a, T> Evaluate<'a, T, f32> for ExprRate
// where
//     T: ExprSelector,
// {
//     fn eval(&'a mut self, metrics: &T) -> ExprResult<'a, f32> {
//         let result = self.inner.eval(metrics)?;

//         match result.extract::<f32>() {
//             Some(value) => {
//                 if (0.0..=1.0).contains(&value) {
//                     Ok(value)
//                 } else {
//                     Err(radiate_err!(
//                         Expr: "Expected f32 value between 0.0 and 1.0 from expression evaluation, but got {}",
//                         value
//                     ))
//                 }
//             }
//             None => Err(radiate_err!(
//                 Expr: "Expected f32 value from expression evaluation, but got {:?}",
//                 result
//             )),
//         }
//     }
// }
