use crate::{AnyValue, Expr, ExprPath};

// pub trait ExprNode {
//     type Output;
//     fn visit<F>(&mut self, f: &mut F)
//     where
//         F: for<'b> FnMut(ExprPath<'_>, LeafView<'b, T>);
// }

// impl<'a> ExprNode<AnyValue<'a>> for AnyValue<'a> {
//     fn visit<F>(&mut self, f: &mut F)
//     where
//         F: for<'b> FnMut(ExprPath<'_>, LeafView<'b, AnyValue<'a>>),
//     {
//         match self {
//             AnyValue::Struct(pairs) => {
//                 for (v, _) in pairs.iter_mut() {
//                     v.visit(f);
//                 }
//             }
//             AnyValue::Vector(vec) => {
//                 for v in vec.iter_mut() {
//                     v.visit(f);
//                 }
//             }
//             _ => {
//                 f(ExprPath::Root, LeafView { value: self });
//             }
//         }
//     }
// }

// pub struct PairLeafView<'x, T> {
//     pub left: &'x mut T,
//     pub right: &'x mut T,
// }

// pub trait ExprNodePair<T> {
//     fn visit_pair<F>(&mut self, f: &mut F)
//     where
//         F: for<'x> FnMut(ExprPath<'x>, PairLeafView<'x, T>);
// }

// impl<'a> ExprNodePair<AnyValue<'a>> for (&mut AnyValue<'a>, &mut AnyValue<'a>) {
//     fn visit_pair<F>(&mut self, f: &mut F)
//     where
//         F: for<'x> FnMut(ExprPath<'x>, PairLeafView<'x, AnyValue<'a>>),
//     {
//         use crate::AnyValue::*;

//         match (&mut self.0, &mut self.1) {
//             (Struct(sa), Struct(sb)) => {
//                 for (va, fa) in sa.iter_mut() {
//                     if let Some((vb, _fb)) = sb.iter_mut().find(|(_, fb)| fb.name() == fa.name()) {
//                         (va, vb).visit_pair(f);
//                     }
//                 }
//             }
//             (Vector(va), Vector(vb)) => {
//                 let n = va.len().min(vb.len());
//                 for i in 0..n {
//                     (&mut va[i], &mut vb[i]).visit_pair(f);
//                 }
//             }
//             (la, lb) => {
//                 f(
//                     ExprPath::Root,
//                     PairLeafView {
//                         left: la,
//                         right: lb,
//                     },
//                 );
//             }
//         }
//     }
// }
