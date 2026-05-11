mod access;
mod defaults;
pub mod expression;
mod fmt;
mod metric;
mod set;
mod tag;
mod view;

pub use defaults::{metric_names, metric_tags};
pub use expression::*;
pub use fmt::{fmt_duration, render_dashboard, render_full, render_metric_rows_full, sparkline};
pub use metric::*;

pub use set::{MetricSet, MetricSetUpdate};
pub use tag::{Tag, TagType};
pub use view::MetricView;

// mod test_expr {
//     pub use super::{Metric, MetricIdx, MetricSet};
//     use radiate_utils::{Slope, SmallStr, Statistic, WindowBuffer};

//     pub enum Expr {
//         Literal(MetricValue),
//         Select(SelectExpr),
//         Rolling(Box<RollingExpr>),
//         Schedule(ScheduleExpr),
//         Binary(Box<BinaryExpr>),
//         Unary(Box<UnaryExpr>),
//         Trinary(Box<TrinaryExpr>),
//     }

//     pub enum SelectExpr {
//         Pending(SmallStr, StatField), // user-built, not yet resolved
//         Bound(MetricIdx, StatField),  // resolved at boot
//     }

//     #[derive(Clone, Copy, Debug, PartialEq)]
//     pub enum StatField {
//         LastValue,
//         Mean,
//         StdDev,
//         Min,
//         Max,
//         Sum,
//         Var,
//         Skew,
//         Count,
//         Generation,
//         UpdateCount,
//         Quantile(f32),
//     }

//     #[derive(Clone, Copy, Debug, PartialEq)]
//     pub enum MetricValue {
//         Null,
//         Bool(bool),
//         F32(f32),
//         U64(u64),
//     }

//     pub struct RollingExpr {
//         child: Expr,
//         rollup: Rollup,
//         buffer: WindowBuffer<f32>, // typed, no AnyValue indirection
//     }

//     #[derive(Clone, Copy, Debug, PartialEq)]
//     pub enum Rollup {
//         First,
//         Last,
//         Mean,
//         StdDev,
//         Min,
//         Max,
//         Sum,
//         Var,
//         Skew,
//         Count,
//         Slope,
//         Quantile(f32),
//     }

//     pub struct EveryState {
//         max: usize,
//         count: usize,
//     }

//     impl EveryState {
//         pub fn new(interval: usize) -> Self {
//             Self {
//                 max: interval,
//                 count: 0,
//             }
//         }

//         pub fn tick(&mut self) -> bool {
//             self.count += 1;
//             if self.count >= self.max {
//                 self.count = 0;
//                 true
//             } else {
//                 false
//             }
//         }

//         pub fn reset(&mut self) {
//             self.count = 0;
//         }
//     }

//     pub enum ScheduleExpr {
//         Every(EveryState), // returns Bool
//     }

//     pub enum TrinaryOp {
//         If,    // (cond, then, else)
//         Clamp, // (value, min, max)
//     }

//     impl MetricValue {
//         pub fn as_f32(&self) -> Option<f32> {
//             match self {
//                 MetricValue::F32(v) => Some(*v),
//                 MetricValue::U64(v) => Some(*v as f32),
//                 MetricValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
//                 MetricValue::Null => None,
//             }
//         }

//         pub fn as_bool(&self) -> Option<bool> {
//             match self {
//                 MetricValue::Bool(b) => Some(*b),
//                 MetricValue::F32(v) => Some(*v != 0.0 && v.is_finite()),
//                 MetricValue::U64(v) => Some(*v != 0),
//                 MetricValue::Null => None,
//             }
//         }

//         pub fn is_finite_number(&self) -> bool {
//             matches!(self, MetricValue::U64(_))
//                 || matches!(self, MetricValue::F32(v) if v.is_finite())
//         }
//     }

//     pub enum BinaryOp {
//         Add,
//         Sub,
//         Mul,
//         Div,
//         Mod,
//         Pow,
//         Min,
//         Max,
//         And,
//         Or,
//         Lt,
//         Lte,
//         Gt,
//         Gte,
//         Eq,
//         Ne,
//         Coalesce,
//     }

//     pub struct BinaryExpr {
//         pub lhs: Expr,
//         pub rhs: Expr,
//         pub op: BinaryOp,
//     }

//     impl BinaryExpr {
//         pub fn eval(&mut self, set: &MetricSet) -> MetricValue {
//             // Coalesce short-circuits — only eval rhs when lhs is unusable.
//             if let BinaryOp::Coalesce = self.op {
//                 let lhs = self.lhs.eval(set);
//                 return if lhs.is_finite_number() || matches!(lhs, MetricValue::Bool(_)) {
//                     lhs
//                 } else {
//                     self.rhs.eval(set)
//                 };
//             }

//             let lhs = self.lhs.eval(set);
//             let rhs = self.rhs.eval(set);

//             match self.op {
//                 BinaryOp::Coalesce => unreachable!(),

//                 BinaryOp::And => MetricValue::Bool(
//                     lhs.as_bool().unwrap_or(false) && rhs.as_bool().unwrap_or(false),
//                 ),
//                 BinaryOp::Or => MetricValue::Bool(
//                     lhs.as_bool().unwrap_or(false) || rhs.as_bool().unwrap_or(false),
//                 ),

//                 BinaryOp::Lt
//                 | BinaryOp::Lte
//                 | BinaryOp::Gt
//                 | BinaryOp::Gte
//                 | BinaryOp::Eq
//                 | BinaryOp::Ne => {
//                     let (Some(a), Some(b)) = (lhs.as_f32(), rhs.as_f32()) else {
//                         return MetricValue::Null;
//                     };
//                     MetricValue::Bool(match self.op {
//                         BinaryOp::Lt => a < b,
//                         BinaryOp::Lte => a <= b,
//                         BinaryOp::Gt => a > b,
//                         BinaryOp::Gte => a >= b,
//                         BinaryOp::Eq => a == b,
//                         BinaryOp::Ne => a != b,
//                         _ => unreachable!(),
//                     })
//                 }

//                 BinaryOp::Add
//                 | BinaryOp::Sub
//                 | BinaryOp::Mul
//                 | BinaryOp::Div
//                 | BinaryOp::Mod
//                 | BinaryOp::Pow
//                 | BinaryOp::Min
//                 | BinaryOp::Max => {
//                     let (Some(a), Some(b)) = (lhs.as_f32(), rhs.as_f32()) else {
//                         return MetricValue::Null;
//                     };
//                     MetricValue::F32(match self.op {
//                         BinaryOp::Add => a + b,
//                         BinaryOp::Sub => a - b,
//                         BinaryOp::Mul => a * b,
//                         BinaryOp::Div => a / b,
//                         BinaryOp::Mod => a % b,
//                         BinaryOp::Pow => a.powf(b),
//                         BinaryOp::Min => a.min(b),
//                         BinaryOp::Max => a.max(b),
//                         _ => unreachable!(),
//                     })
//                 }
//             }
//         }

//         pub fn reset(&mut self) {
//             self.lhs.reset();
//             self.rhs.reset();
//         }
//     }

//     #[derive(Clone, Copy, Debug, PartialEq)]
//     pub enum MetricKind {
//         Bool,
//         F32,
//         U64,
//     }

//     pub enum UnaryOp {
//         Neg,
//         Abs,
//         Not,
//         IsNull,
//         IsFinite,
//         IsNan,
//         Cast(MetricKind),
//     }

//     pub struct UnaryExpr {
//         pub child: Expr,
//         pub op: UnaryOp,
//     }

//     impl UnaryExpr {
//         pub fn eval(&mut self, set: &MetricSet) -> MetricValue {
//             let v = self.child.eval(set);
//             match self.op {
//                 UnaryOp::Neg => match v.as_f32() {
//                     Some(x) => MetricValue::F32(-x),
//                     None => MetricValue::Null,
//                 },
//                 UnaryOp::Abs => match v.as_f32() {
//                     Some(x) => MetricValue::F32(x.abs()),
//                     None => MetricValue::Null,
//                 },
//                 UnaryOp::Not => MetricValue::Bool(!v.as_bool().unwrap_or(false)),

//                 UnaryOp::IsNull => MetricValue::Bool(matches!(v, MetricValue::Null)),
//                 UnaryOp::IsFinite => MetricValue::Bool(v.is_finite_number()),
//                 UnaryOp::IsNan => MetricValue::Bool(matches!(v, MetricValue::F32(x) if x.is_nan())),

//                 UnaryOp::Cast(kind) => match kind {
//                     MetricKind::F32 => v
//                         .as_f32()
//                         .map(MetricValue::F32)
//                         .unwrap_or(MetricValue::Null),
//                     MetricKind::Bool => v
//                         .as_bool()
//                         .map(MetricValue::Bool)
//                         .unwrap_or(MetricValue::Null),
//                     MetricKind::U64 => v
//                         .as_f32()
//                         .map(|x| {
//                             if x.is_finite() {
//                                 MetricValue::U64(x.max(0.0) as u64)
//                             } else {
//                                 MetricValue::Null
//                             }
//                         })
//                         .unwrap_or(MetricValue::Null),
//                 },
//             }
//         }

//         pub fn reset(&mut self) {
//             self.child.reset();
//         }
//     }

//     pub struct TrinaryExpr {
//         pub a: Expr,
//         pub b: Expr,
//         pub c: Expr,
//         pub op: TrinaryOp,
//     }

//     impl TrinaryExpr {
//         pub fn eval(&mut self, set: &MetricSet) -> MetricValue {
//             match self.op {
//                 TrinaryOp::If => {
//                     // Lazy eval — only the taken branch advances stateful children.
//                     let cond = self.a.eval(set);
//                     if cond.as_bool().unwrap_or(false) {
//                         self.b.eval(set)
//                     } else {
//                         self.c.eval(set)
//                     }
//                 }

//                 TrinaryOp::Clamp => {
//                     let v = self.a.eval(set).as_f32();
//                     let lo = self.b.eval(set).as_f32();
//                     let hi = self.c.eval(set).as_f32();
//                     let (Some(lo), Some(hi)) = (lo, hi) else {
//                         return MetricValue::Null;
//                     };
//                     // Null / NaN / Inf in v falls back to floor — conservative default
//                     // for rate-style controllers.
//                     let out = match v {
//                         Some(x) if x.is_finite() => x.clamp(lo, hi),
//                         _ => lo,
//                     };
//                     MetricValue::F32(out)
//                 }
//             }
//         }

//         pub fn reset(&mut self) {
//             self.a.reset();
//             self.b.reset();
//             self.c.reset();
//         }
//     }

//     // ---------------------------------------------------------------------------
//     // Expr dispatch
//     // ---------------------------------------------------------------------------

//     impl Expr {
//         pub fn eval(&mut self, set: &MetricSet) -> MetricValue {
//             match self {
//                 Expr::Literal(v) => *v,
//                 Expr::Select(s) => s.eval(set),
//                 Expr::Rolling(r) => r.eval(set),
//                 Expr::Schedule(s) => s.eval(),
//                 Expr::Binary(b) => b.eval(set),
//                 Expr::Unary(u) => u.eval(set),
//                 Expr::Trinary(t) => t.eval(set),
//             }
//         }

//         pub fn reset(&mut self) {
//             match self {
//                 Expr::Literal(_) | Expr::Select(_) => {}
//                 Expr::Rolling(r) => r.reset(),
//                 Expr::Schedule(s) => s.reset(),
//                 Expr::Binary(b) => b.reset(),
//                 Expr::Unary(u) => u.reset(),
//                 Expr::Trinary(t) => t.reset(),
//             }
//         }
//     }

//     // MetricValue needs to be Copy for `Expr::Literal(v) => *v` above.
//     // Add `#[derive(Clone, Copy, Debug, PartialEq)]` to MetricValue.

//     // ---------------------------------------------------------------------------
//     // SelectExpr
//     // ---------------------------------------------------------------------------

//     impl SelectExpr {
//         pub fn eval(&self, set: &MetricSet) -> MetricValue {
//             match self {
//                 SelectExpr::Pending(_, _) => {
//                     debug_assert!(
//                         false,
//                         "Pending SelectExpr at eval time — call Expr::resolve first"
//                     );
//                     MetricValue::Null
//                 }
//                 SelectExpr::Bound(idx, field) => {
//                     let Some(m) = set.get_by_idx(*idx) else {
//                         return MetricValue::Null;
//                     };
//                     read_metric_field(m, *field)
//                 }
//             }
//         }
//     }

//     fn read_metric_field(m: &Metric, field: StatField) -> MetricValue {
//         match field {
//             StatField::LastValue => MetricValue::F32(m.last_value()),
//             StatField::Mean => MetricValue::F32(m.mean()),
//             StatField::StdDev => MetricValue::F32(m.stddev()),
//             StatField::Min => MetricValue::F32(m.min()),
//             StatField::Max => MetricValue::F32(m.max()),
//             StatField::Sum => MetricValue::F32(m.sum()),
//             StatField::Var => MetricValue::F32(m.var()),
//             StatField::Skew => MetricValue::F32(m.skew()),
//             StatField::Count => MetricValue::U64(m.count() as u64),
//             StatField::Generation => MetricValue::U64(m.generation()),
//             StatField::UpdateCount => MetricValue::U64(m.update_count() as u64),
//             StatField::Quantile(_q) => {
//                 // TODO: once Metric grows quantile rollups, return
//                 //   m.quantile(_q).map(MetricValue::F32).unwrap_or(MetricValue::Null)
//                 MetricValue::Null
//             }
//         }
//     }

//     // ---------------------------------------------------------------------------
//     // ScheduleExpr
//     // ---------------------------------------------------------------------------

//     impl ScheduleExpr {
//         pub fn eval(&mut self) -> MetricValue {
//             match self {
//                 // EveryState currently has private `count`/`max` and only `pub(super)`
//                 // reset. Either:
//                 //   - add `pub fn tick(&mut self) -> bool` to EveryState (preferred)
//                 //   - widen field visibility to pub(crate)
//                 //   - or replace EveryState with a fresh one local to this module
//                 ScheduleExpr::Every(state) => MetricValue::Bool(state.tick()),
//             }
//         }

//         pub fn reset(&mut self) {
//             match self {
//                 // Needs pub fn reset on EveryState (currently pub(super)).
//                 ScheduleExpr::Every(state) => state.reset(),
//             }
//         }
//     }

//     // ---------------------------------------------------------------------------
//     // RollingExpr
//     // ---------------------------------------------------------------------------

//     impl RollingExpr {
//         pub fn eval(&mut self, set: &MetricSet) -> MetricValue {
//             let child = self.child.eval(set);
//             let Some(x) = child.as_f32() else {
//                 return MetricValue::Null;
//             };
//             self.buffer.push(x);
//             compute_rollup(self.buffer.values(), self.rollup)
//         }

//         pub fn reset(&mut self) {
//             self.buffer.clear();
//             self.child.reset();
//         }
//     }

//     fn compute_rollup(values: &[f32], rollup: Rollup) -> MetricValue {
//         if values.is_empty() {
//             return match rollup {
//                 Rollup::Count => MetricValue::U64(0),
//                 _ => MetricValue::Null,
//             };
//         }

//         if values.len() == 1 {
//             return match rollup {
//                 Rollup::Count => MetricValue::U64(1),
//                 Rollup::First
//                 | Rollup::Last
//                 | Rollup::Min
//                 | Rollup::Max
//                 | Rollup::Mean
//                 | Rollup::Sum => MetricValue::F32(values[0]),
//                 Rollup::StdDev | Rollup::Var | Rollup::Skew | Rollup::Slope => {
//                     MetricValue::F32(0.0)
//                 }
//                 Rollup::Quantile(_) => MetricValue::F32(values[0]),
//             };
//         }

//         match rollup {
//             Rollup::Count => MetricValue::U64(values.len() as u64),
//             Rollup::First => MetricValue::F32(values[0]),
//             Rollup::Last => MetricValue::F32(values[values.len() - 1]),
//             Rollup::Sum => MetricValue::F32(values.iter().sum()),
//             Rollup::Mean => MetricValue::F32(values.iter().sum::<f32>() / values.len() as f32),
//             Rollup::Min => MetricValue::F32(values.iter().copied().fold(f32::INFINITY, f32::min)),
//             Rollup::Max => {
//                 MetricValue::F32(values.iter().copied().fold(f32::NEG_INFINITY, f32::max))
//             }
//             Rollup::StdDev => {
//                 let stat: Statistic = values.iter().copied().collect();
//                 MetricValue::F32(stat.std_dev().unwrap_or(0.0))
//             }
//             Rollup::Var => {
//                 let stat: Statistic = values.iter().copied().collect();
//                 MetricValue::F32(stat.variance().unwrap_or(0.0))
//             }
//             Rollup::Skew => {
//                 let stat: Statistic = values.iter().copied().collect();
//                 MetricValue::F32(stat.skewness().unwrap_or(0.0))
//             }
//             Rollup::Slope => {
//                 let slope: Slope<f32> = values.iter().copied().collect();
//                 MetricValue::F32(slope.value().unwrap_or(0.0))
//             }
//             Rollup::Quantile(q) => {
//                 // Sort + linear interpolation. P² estimator doesn't apply here —
//                 // rolling windows are sliding, P² has no remove operation.
//                 let mut sorted: Vec<f32> =
//                     values.iter().copied().filter(|v| v.is_finite()).collect();
//                 if sorted.is_empty() {
//                     return MetricValue::F32(0.0);
//                 }
//                 sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
//                 let q = q.clamp(0.0, 1.0);
//                 let pos = q * (sorted.len() - 1) as f32;
//                 let lo = pos.floor() as usize;
//                 let hi = pos.ceil() as usize;
//                 let frac = pos - lo as f32;
//                 MetricValue::F32(sorted[lo] * (1.0 - frac) + sorted[hi] * frac)
//             }
//         }
//     }

//     // -----------------------------------------------------------------------
//     // Builder DSL — mirrors radiate-core/src/stats/expression/builder.rs
//     // for the new metric-focused Expr.
//     // -----------------------------------------------------------------------

//     // From-conversion helpers — let literals coerce into Expr in builder chains.

//     impl From<f32> for MetricValue {
//         fn from(v: f32) -> Self {
//             MetricValue::F32(v)
//         }
//     }
//     impl From<bool> for MetricValue {
//         fn from(v: bool) -> Self {
//             MetricValue::Bool(v)
//         }
//     }
//     impl From<u64> for MetricValue {
//         fn from(v: u64) -> Self {
//             MetricValue::U64(v)
//         }
//     }

//     impl From<f32> for Expr {
//         fn from(v: f32) -> Self {
//             Expr::Literal(MetricValue::F32(v))
//         }
//     }
//     impl From<f64> for Expr {
//         fn from(v: f64) -> Self {
//             Expr::Literal(MetricValue::F32(v as f32))
//         }
//     }
//     impl From<i32> for Expr {
//         fn from(v: i32) -> Self {
//             Expr::Literal(MetricValue::F32(v as f32))
//         }
//     }
//     impl From<u64> for Expr {
//         fn from(v: u64) -> Self {
//             Expr::Literal(MetricValue::U64(v))
//         }
//     }
//     impl From<usize> for Expr {
//         fn from(v: usize) -> Self {
//             Expr::Literal(MetricValue::U64(v as u64))
//         }
//     }
//     impl From<bool> for Expr {
//         fn from(v: bool) -> Self {
//             Expr::Literal(MetricValue::Bool(v))
//         }
//     }

//     // Free constructors — mirrors the old `expr::` module.

//     pub mod expr {
//         use super::*;
//         use radiate_utils::SmallStr;

//         pub fn lit(v: impl Into<MetricValue>) -> Expr {
//             Expr::Literal(v.into())
//         }

//         pub fn select(name: impl Into<SmallStr>) -> Expr {
//             Expr::Select(SelectExpr::Pending(name.into(), StatField::LastValue))
//         }

//         pub fn every(interval: usize) -> When {
//             When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
//                 interval,
//             ))))
//         }

//         pub fn when(cond: impl Into<Expr>) -> When {
//             When::new(cond.into())
//         }
//     }

//     // Conditional / schedule builder: `when(c).then(x).otherwise(y)`.

//     pub struct When {
//         cond: Expr,
//     }
//     impl When {
//         pub fn new(cond: impl Into<Expr>) -> Self {
//             Self { cond: cond.into() }
//         }
//         pub fn then(self, then_expr: impl Into<Expr>) -> Then {
//             Then {
//                 cond: self.cond,
//                 then_expr: then_expr.into(),
//             }
//         }
//     }

//     pub struct Then {
//         cond: Expr,
//         then_expr: Expr,
//     }
//     impl Then {
//         pub fn otherwise(self, else_expr: impl Into<Expr>) -> Expr {
//             Expr::Trinary(Box::new(TrinaryExpr {
//                 a: self.cond,
//                 b: self.then_expr,
//                 c: else_expr.into(),
//                 op: TrinaryOp::If,
//             }))
//         }
//     }

//     // Private construction helpers — keep call sites flat.

//     fn make_bin(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
//         Expr::Binary(Box::new(BinaryExpr { lhs, rhs, op }))
//     }
//     fn make_un(op: UnaryOp, child: Expr) -> Expr {
//         Expr::Unary(Box::new(UnaryExpr { child, op }))
//     }

//     // Chainable methods on Expr.

//     impl Expr {
//         // Set the StatField for a Select, or the Rollup for a Rolling, in-place.
//         // For other variants, this is a passthrough — "mean of a binary expr" isn't
//         // semantically defined in this model.
//         fn with_stat(self, field: StatField, rollup: Rollup) -> Expr {
//             match self {
//                 Expr::Select(SelectExpr::Pending(name, _)) => {
//                     Expr::Select(SelectExpr::Pending(name, field))
//                 }
//                 Expr::Select(SelectExpr::Bound(idx, _)) => {
//                     Expr::Select(SelectExpr::Bound(idx, field))
//                 }
//                 Expr::Rolling(mut r) => {
//                     r.rollup = rollup;
//                     Expr::Rolling(r)
//                 }
//                 other => other,
//             }
//         }

//         // ---- aggregate field selectors ----
//         pub fn last_value(self) -> Expr {
//             self.with_stat(StatField::LastValue, Rollup::Last)
//         }
//         pub fn mean(self) -> Expr {
//             self.with_stat(StatField::Mean, Rollup::Mean)
//         }
//         pub fn stddev(self) -> Expr {
//             self.with_stat(StatField::StdDev, Rollup::StdDev)
//         }
//         pub fn min(self) -> Expr {
//             self.with_stat(StatField::Min, Rollup::Min)
//         }
//         pub fn max(self) -> Expr {
//             self.with_stat(StatField::Max, Rollup::Max)
//         }
//         pub fn sum(self) -> Expr {
//             self.with_stat(StatField::Sum, Rollup::Sum)
//         }
//         pub fn var(self) -> Expr {
//             self.with_stat(StatField::Var, Rollup::Var)
//         }
//         pub fn skew(self) -> Expr {
//             self.with_stat(StatField::Skew, Rollup::Skew)
//         }
//         pub fn count(self) -> Expr {
//             self.with_stat(StatField::Count, Rollup::Count)
//         }
//         pub fn quantile(self, q: f32) -> Expr {
//             self.with_stat(StatField::Quantile(q), Rollup::Quantile(q))
//         }

//         // ---- rolling-only rollups (no StatField equivalent) ----
//         fn with_rollup(self, rollup: Rollup) -> Expr {
//             match self {
//                 Expr::Rolling(mut r) => {
//                     r.rollup = rollup;
//                     Expr::Rolling(r)
//                 }
//                 other => other,
//             }
//         }
//         pub fn first(self) -> Expr {
//             self.with_rollup(Rollup::First)
//         }
//         pub fn last(self) -> Expr {
//             self.with_rollup(Rollup::Last)
//         }
//         pub fn slope(self) -> Expr {
//             self.with_rollup(Rollup::Slope)
//         }

//         /// Wrap in a sliding-window rolling aggregate. If `self` is already a
//         /// `Rolling`, replaces its buffer; otherwise wraps with `Rollup::Last`.
//         pub fn rolling(self, window: usize) -> Expr {
//             match self {
//                 Expr::Rolling(mut r) => {
//                     r.buffer = WindowBuffer::with_capacity(window);
//                     Expr::Rolling(r)
//                 }
//                 other => Expr::Rolling(Box::new(RollingExpr {
//                     child: other,
//                     rollup: Rollup::Last,
//                     buffer: WindowBuffer::with_capacity(window),
//                 })),
//             }
//         }

//         // ---- arithmetic ----
//         #[allow(clippy::should_implement_trait)]
//         pub fn add(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Add, rhs.into())
//         }
//         #[allow(clippy::should_implement_trait)]
//         pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Sub, rhs.into())
//         }
//         #[allow(clippy::should_implement_trait)]
//         pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Mul, rhs.into())
//         }
//         #[allow(clippy::should_implement_trait)]
//         pub fn div(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Div, rhs.into())
//         }
//         pub fn modulo(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Mod, rhs.into())
//         }
//         pub fn pow(self, exp: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Pow, exp.into())
//         }

//         // ---- unary ----
//         #[allow(clippy::should_implement_trait)]
//         pub fn neg(self) -> Expr {
//             make_un(UnaryOp::Neg, self)
//         }
//         pub fn abs(self) -> Expr {
//             make_un(UnaryOp::Abs, self)
//         }
//         #[allow(clippy::should_implement_trait)]
//         pub fn not(self) -> Expr {
//             make_un(UnaryOp::Not, self)
//         }
//         pub fn is_null(self) -> Expr {
//             make_un(UnaryOp::IsNull, self)
//         }
//         pub fn is_finite(self) -> Expr {
//             make_un(UnaryOp::IsFinite, self)
//         }
//         pub fn is_nan(self) -> Expr {
//             make_un(UnaryOp::IsNan, self)
//         }
//         pub fn cast(self, kind: MetricKind) -> Expr {
//             make_un(UnaryOp::Cast(kind), self)
//         }

//         // ---- comparisons ----
//         pub fn lt(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Lt, rhs.into())
//         }
//         pub fn lte(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Lte, rhs.into())
//         }
//         pub fn gt(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Gt, rhs.into())
//         }
//         pub fn gte(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Gte, rhs.into())
//         }
//         #[allow(clippy::should_implement_trait)]
//         pub fn eq(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Eq, rhs.into())
//         }
//         pub fn ne(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Ne, rhs.into())
//         }

//         // ---- logical ----
//         pub fn and(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::And, rhs.into())
//         }
//         pub fn or(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Or, rhs.into())
//         }

//         // ---- elementwise min/max ----
//         /// `min(self, rhs)` — use as a ceiling.
//         pub fn min_with(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Min, rhs.into())
//         }
//         /// `max(self, rhs)` — use as a floor.
//         pub fn max_with(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Max, rhs.into())
//         }

//         /// Returns `self` if it evaluates to a finite number or bool, otherwise `rhs`.
//         /// Short-circuits — `rhs` is only evaluated when needed.
//         pub fn or_else(self, rhs: impl Into<Expr>) -> Expr {
//             make_bin(self, BinaryOp::Coalesce, rhs.into())
//         }

//         /// Clamp `self` to `[min, max]`. Null/NaN/Inf falls to `min` (conservative
//         /// default for rate-style controllers).
//         pub fn clamp(self, min: impl Into<Expr>, max: impl Into<Expr>) -> Expr {
//             Expr::Trinary(Box::new(TrinaryExpr {
//                 a: self,
//                 b: min.into(),
//                 c: max.into(),
//                 op: TrinaryOp::Clamp,
//             }))
//         }

//         /// Walks the tree, resolving every `Pending` selector against `set`.
//         /// Returns `true` if every selector was resolved.
//         pub fn resolve(&mut self, set: &MetricSet) -> bool {
//             match self {
//                 Expr::Select(s) => s.resolve(set),
//                 Expr::Literal(_) | Expr::Schedule(_) => true,
//                 Expr::Rolling(r) => r.child.resolve(set),
//                 Expr::Binary(b) => {
//                     let l = b.lhs.resolve(set);
//                     let r = b.rhs.resolve(set);
//                     l && r
//                 }
//                 Expr::Unary(u) => u.child.resolve(set),
//                 Expr::Trinary(t) => {
//                     let a = t.a.resolve(set);
//                     let b = t.b.resolve(set);
//                     let c = t.c.resolve(set);
//                     a && b && c
//                 }
//             }
//         }
//     }

//     impl SelectExpr {
//         /// Resolves a `Pending` selector against `set`. Returns `true` on success
//         /// (or if already `Bound`). Returns `false` for an unknown metric name.
//         pub fn resolve(&mut self, set: &MetricSet) -> bool {
//             match self {
//                 SelectExpr::Bound(_, _) => true,
//                 SelectExpr::Pending(name, field) => match set.get_idx(name.as_str()) {
//                     Some(idx) => {
//                         *self = SelectExpr::Bound(idx, *field);
//                         true
//                     }
//                     None => false,
//                 },
//             }
//         }
//     }

//     // ---- operator overloads ----

//     impl std::ops::Add for Expr {
//         type Output = Expr;
//         fn add(self, rhs: Expr) -> Expr {
//             make_bin(self, BinaryOp::Add, rhs)
//         }
//     }
//     impl std::ops::Sub for Expr {
//         type Output = Expr;
//         fn sub(self, rhs: Expr) -> Expr {
//             make_bin(self, BinaryOp::Sub, rhs)
//         }
//     }
//     impl std::ops::Mul for Expr {
//         type Output = Expr;
//         fn mul(self, rhs: Expr) -> Expr {
//             make_bin(self, BinaryOp::Mul, rhs)
//         }
//     }
//     impl std::ops::Div for Expr {
//         type Output = Expr;
//         fn div(self, rhs: Expr) -> Expr {
//             make_bin(self, BinaryOp::Div, rhs)
//         }
//     }
//     impl std::ops::Neg for Expr {
//         type Output = Expr;
//         fn neg(self) -> Expr {
//             make_un(UnaryOp::Neg, self)
//         }
//     }
//     impl std::ops::Not for Expr {
//         type Output = Expr;
//         fn not(self) -> Expr {
//             make_un(UnaryOp::Not, self)
//         }
//     }

//     // -----------------------------------------------------------------------
//     // tests
//     // -----------------------------------------------------------------------

//     #[cfg(test)]
//     mod tests {
//         use super::*;

//         const EPS: f32 = 1e-5;

//         // Small construction helpers — these keep the test bodies readable.
//         fn lit_f32(v: f32) -> Expr {
//             Expr::Literal(MetricValue::F32(v))
//         }
//         fn lit_bool(b: bool) -> Expr {
//             Expr::Literal(MetricValue::Bool(b))
//         }
//         fn lit_u64(v: u64) -> Expr {
//             Expr::Literal(MetricValue::U64(v))
//         }
//         fn lit_null() -> Expr {
//             Expr::Literal(MetricValue::Null)
//         }
//         fn bin(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
//             Expr::Binary(Box::new(BinaryExpr { lhs, rhs, op }))
//         }
//         fn un(op: UnaryOp, child: Expr) -> Expr {
//             Expr::Unary(Box::new(UnaryExpr { child, op }))
//         }
//         fn tri(op: TrinaryOp, a: Expr, b: Expr, c: Expr) -> Expr {
//             Expr::Trinary(Box::new(TrinaryExpr { a, b, c, op }))
//         }
//         fn rolling(child: Expr, rollup: Rollup, window: usize) -> Expr {
//             Expr::Rolling(Box::new(RollingExpr {
//                 child,
//                 rollup,
//                 buffer: WindowBuffer::with_capacity(window),
//             }))
//         }
//         fn approx(a: f32, b: f32) -> bool {
//             (a - b).abs() < EPS
//         }
//         fn unwrap_f32(v: MetricValue) -> f32 {
//             v.as_f32().expect("expected f32-coercible MetricValue")
//         }
//         fn unwrap_bool(v: MetricValue) -> bool {
//             v.as_bool().expect("expected bool-coercible MetricValue")
//         }
//         fn empty_set() -> MetricSet {
//             MetricSet::default()
//         }

//         // -- MetricValue helpers ------------------------------------------------

//         #[test]
//         fn metric_value_as_f32_coerces() {
//             assert_eq!(MetricValue::F32(1.5).as_f32(), Some(1.5));
//             assert_eq!(MetricValue::U64(7).as_f32(), Some(7.0));
//             assert_eq!(MetricValue::Bool(true).as_f32(), Some(1.0));
//             assert_eq!(MetricValue::Bool(false).as_f32(), Some(0.0));
//             assert_eq!(MetricValue::Null.as_f32(), None);
//         }

//         #[test]
//         fn metric_value_as_bool_coerces() {
//             assert_eq!(MetricValue::Bool(true).as_bool(), Some(true));
//             assert_eq!(MetricValue::Bool(false).as_bool(), Some(false));
//             assert_eq!(MetricValue::F32(0.0).as_bool(), Some(false));
//             assert_eq!(MetricValue::F32(1.5).as_bool(), Some(true));
//             assert_eq!(MetricValue::F32(f32::NAN).as_bool(), Some(false));
//             assert_eq!(MetricValue::U64(0).as_bool(), Some(false));
//             assert_eq!(MetricValue::U64(5).as_bool(), Some(true));
//             assert_eq!(MetricValue::Null.as_bool(), None);
//         }

//         #[test]
//         fn metric_value_is_finite_number() {
//             assert!(MetricValue::F32(1.0).is_finite_number());
//             assert!(MetricValue::U64(42).is_finite_number());
//             assert!(!MetricValue::F32(f32::INFINITY).is_finite_number());
//             assert!(!MetricValue::F32(f32::NAN).is_finite_number());
//             assert!(!MetricValue::Bool(true).is_finite_number());
//             assert!(!MetricValue::Null.is_finite_number());
//         }

//         // -- EveryState ----------------------------------------------------------

//         #[test]
//         fn every_state_fires_at_interval() {
//             let mut s = EveryState::new(3);
//             assert!(!s.tick());
//             assert!(!s.tick());
//             assert!(s.tick());
//             assert!(!s.tick());
//             assert!(!s.tick());
//             assert!(s.tick());
//         }

//         #[test]
//         fn every_state_reset_restarts_counter() {
//             let mut s = EveryState::new(3);
//             s.tick();
//             s.tick();
//             s.reset();
//             assert!(!s.tick());
//             assert!(!s.tick());
//             assert!(s.tick());
//         }

//         // -- BinaryExpr ----------------------------------------------------------

//         #[test]
//         fn binary_arithmetic_on_f32() {
//             let set = empty_set();
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(2.0), BinaryOp::Add, lit_f32(3.0)).eval(&set)),
//                 5.0
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(5.0), BinaryOp::Sub, lit_f32(2.0)).eval(&set)),
//                 3.0
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(4.0), BinaryOp::Mul, lit_f32(3.0)).eval(&set)),
//                 12.0
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(10.0), BinaryOp::Div, lit_f32(4.0)).eval(&set)),
//                 2.5
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(10.0), BinaryOp::Mod, lit_f32(3.0)).eval(&set)),
//                 1.0
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(2.0), BinaryOp::Pow, lit_f32(8.0)).eval(&set)),
//                 256.0
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(1.0), BinaryOp::Min, lit_f32(5.0)).eval(&set)),
//                 1.0
//             ));
//             assert!(approx(
//                 unwrap_f32(bin(lit_f32(1.0), BinaryOp::Max, lit_f32(5.0)).eval(&set)),
//                 5.0
//             ));
//         }

//         #[test]
//         fn binary_coerces_u64_and_bool_to_f32() {
//             let set = empty_set();
//             // U64 + F32 → F32
//             assert!(approx(
//                 unwrap_f32(bin(lit_u64(3), BinaryOp::Add, lit_f32(1.5)).eval(&set)),
//                 4.5
//             ));
//             // Bool + F32 → F32 (true == 1.0)
//             assert!(approx(
//                 unwrap_f32(bin(lit_bool(true), BinaryOp::Add, lit_f32(2.0)).eval(&set)),
//                 3.0
//             ));
//         }

//         #[test]
//         fn binary_div_by_zero_is_infinity_not_null() {
//             let set = empty_set();
//             let v = unwrap_f32(bin(lit_f32(1.0), BinaryOp::Div, lit_f32(0.0)).eval(&set));
//             assert!(v.is_infinite());
//         }

//         #[test]
//         fn binary_comparison_returns_bool() {
//             let set = empty_set();
//             assert!(unwrap_bool(
//                 bin(lit_f32(1.0), BinaryOp::Lt, lit_f32(2.0)).eval(&set)
//             ));
//             assert!(!unwrap_bool(
//                 bin(lit_f32(2.0), BinaryOp::Lt, lit_f32(1.0)).eval(&set)
//             ));
//             assert!(unwrap_bool(
//                 bin(lit_f32(1.0), BinaryOp::Eq, lit_f32(1.0)).eval(&set)
//             ));
//             assert!(unwrap_bool(
//                 bin(lit_f32(1.0), BinaryOp::Ne, lit_f32(2.0)).eval(&set)
//             ));
//         }

//         #[test]
//         fn binary_comparison_with_null_returns_null() {
//             let set = empty_set();
//             let v = bin(lit_f32(1.0), BinaryOp::Lt, lit_null()).eval(&set);
//             assert_eq!(v, MetricValue::Null);
//         }

//         #[test]
//         fn binary_arithmetic_with_null_returns_null() {
//             let set = empty_set();
//             let v = bin(lit_f32(1.0), BinaryOp::Add, lit_null()).eval(&set);
//             assert_eq!(v, MetricValue::Null);
//         }

//         #[test]
//         fn binary_logical_and_or() {
//             let set = empty_set();
//             assert!(unwrap_bool(
//                 bin(lit_bool(true), BinaryOp::And, lit_bool(true)).eval(&set)
//             ));
//             assert!(!unwrap_bool(
//                 bin(lit_bool(true), BinaryOp::And, lit_bool(false)).eval(&set)
//             ));
//             assert!(unwrap_bool(
//                 bin(lit_bool(false), BinaryOp::Or, lit_bool(true)).eval(&set)
//             ));
//             // Null treated as false in boolean context
//             assert!(!unwrap_bool(
//                 bin(lit_null(), BinaryOp::And, lit_bool(true)).eval(&set)
//             ));
//             assert!(unwrap_bool(
//                 bin(lit_null(), BinaryOp::Or, lit_bool(true)).eval(&set)
//             ));
//         }

//         #[test]
//         fn binary_coalesce_passes_through_finite() {
//             let set = empty_set();
//             let v = unwrap_f32(bin(lit_f32(3.0), BinaryOp::Coalesce, lit_f32(99.0)).eval(&set));
//             assert!(approx(v, 3.0));
//         }

//         #[test]
//         fn binary_coalesce_falls_back_on_null() {
//             let set = empty_set();
//             let v = unwrap_f32(bin(lit_null(), BinaryOp::Coalesce, lit_f32(42.0)).eval(&set));
//             assert!(approx(v, 42.0));
//         }

//         #[test]
//         fn binary_coalesce_falls_back_on_nan() {
//             let set = empty_set();
//             let v = unwrap_f32(bin(lit_f32(f32::NAN), BinaryOp::Coalesce, lit_f32(7.0)).eval(&set));
//             assert!(approx(v, 7.0));
//         }

//         // -- UnaryExpr -----------------------------------------------------------

//         #[test]
//         fn unary_neg_and_abs() {
//             let set = empty_set();
//             assert!(approx(
//                 unwrap_f32(un(UnaryOp::Neg, lit_f32(3.0)).eval(&set)),
//                 -3.0
//             ));
//             assert!(approx(
//                 unwrap_f32(un(UnaryOp::Abs, lit_f32(-3.0)).eval(&set)),
//                 3.0
//             ));
//         }

//         #[test]
//         fn unary_neg_of_null_is_null() {
//             let set = empty_set();
//             assert_eq!(un(UnaryOp::Neg, lit_null()).eval(&set), MetricValue::Null);
//         }

//         #[test]
//         fn unary_not_inverts_bool() {
//             let set = empty_set();
//             assert_eq!(
//                 un(UnaryOp::Not, lit_bool(true)).eval(&set),
//                 MetricValue::Bool(false)
//             );
//             assert_eq!(
//                 un(UnaryOp::Not, lit_bool(false)).eval(&set),
//                 MetricValue::Bool(true)
//             );
//             // Null → bool default false, not(false) = true
//             assert_eq!(
//                 un(UnaryOp::Not, lit_null()).eval(&set),
//                 MetricValue::Bool(true)
//             );
//         }

//         #[test]
//         fn unary_predicates() {
//             let set = empty_set();
//             assert_eq!(
//                 un(UnaryOp::IsNull, lit_null()).eval(&set),
//                 MetricValue::Bool(true)
//             );
//             assert_eq!(
//                 un(UnaryOp::IsNull, lit_f32(1.0)).eval(&set),
//                 MetricValue::Bool(false)
//             );

//             assert_eq!(
//                 un(UnaryOp::IsFinite, lit_f32(1.0)).eval(&set),
//                 MetricValue::Bool(true)
//             );
//             assert_eq!(
//                 un(UnaryOp::IsFinite, lit_f32(f32::INFINITY)).eval(&set),
//                 MetricValue::Bool(false)
//             );
//             assert_eq!(
//                 un(UnaryOp::IsFinite, lit_null()).eval(&set),
//                 MetricValue::Bool(false)
//             );

//             assert_eq!(
//                 un(UnaryOp::IsNan, lit_f32(f32::NAN)).eval(&set),
//                 MetricValue::Bool(true)
//             );
//             assert_eq!(
//                 un(UnaryOp::IsNan, lit_f32(1.0)).eval(&set),
//                 MetricValue::Bool(false)
//             );
//         }

//         #[test]
//         fn unary_cast() {
//             let set = empty_set();
//             assert_eq!(
//                 un(UnaryOp::Cast(MetricKind::F32), lit_u64(7)).eval(&set),
//                 MetricValue::F32(7.0)
//             );
//             assert_eq!(
//                 un(UnaryOp::Cast(MetricKind::Bool), lit_f32(2.5)).eval(&set),
//                 MetricValue::Bool(true)
//             );
//             assert_eq!(
//                 un(UnaryOp::Cast(MetricKind::U64), lit_f32(3.9)).eval(&set),
//                 MetricValue::U64(3)
//             );
//             // negative → 0
//             assert_eq!(
//                 un(UnaryOp::Cast(MetricKind::U64), lit_f32(-1.0)).eval(&set),
//                 MetricValue::U64(0)
//             );
//             // non-finite → Null
//             assert_eq!(
//                 un(UnaryOp::Cast(MetricKind::U64), lit_f32(f32::INFINITY)).eval(&set),
//                 MetricValue::Null
//             );
//         }

//         // -- TrinaryExpr ---------------------------------------------------------

//         #[test]
//         fn trinary_if_takes_correct_branch() {
//             let set = empty_set();
//             let then_branch = lit_f32(10.0);
//             let else_branch = lit_f32(20.0);
//             assert!(approx(
//                 unwrap_f32(tri(TrinaryOp::If, lit_bool(true), then_branch, else_branch).eval(&set)),
//                 10.0
//             ));

//             let then_branch = lit_f32(10.0);
//             let else_branch = lit_f32(20.0);
//             assert!(approx(
//                 unwrap_f32(
//                     tri(TrinaryOp::If, lit_bool(false), then_branch, else_branch).eval(&set)
//                 ),
//                 20.0
//             ));
//         }

//         #[test]
//         fn trinary_if_null_cond_takes_else() {
//             let set = empty_set();
//             let v = tri(TrinaryOp::If, lit_null(), lit_f32(1.0), lit_f32(2.0)).eval(&set);
//             assert!(approx(unwrap_f32(v), 2.0));
//         }

//         #[test]
//         fn trinary_if_is_lazy_on_schedule_state() {
//             // If the cond is false, the taken (else) branch advances; the then branch must NOT.
//             // Use Schedule::Every to detect: if its tick fired, the count was advanced.
//             let set = empty_set();
//             let then_sched = Expr::Schedule(ScheduleExpr::Every(EveryState::new(2)));
//             let else_sched = Expr::Schedule(ScheduleExpr::Every(EveryState::new(2)));
//             let mut expr = tri(TrinaryOp::If, lit_bool(false), then_sched, else_sched);

//             // Evaluate twice. With lazy eval, only the else's counter advances.
//             // After two evals: else.count = 2 → fires; then.count stays at 0.
//             expr.eval(&set);
//             let second = expr.eval(&set);
//             assert_eq!(
//                 second,
//                 MetricValue::Bool(true),
//                 "else branch should have fired on second eval"
//             );
//         }

//         #[test]
//         fn trinary_clamp_within_range_returns_value() {
//             let set = empty_set();
//             let v = unwrap_f32(
//                 tri(TrinaryOp::Clamp, lit_f32(5.0), lit_f32(0.0), lit_f32(10.0)).eval(&set),
//             );
//             assert!(approx(v, 5.0));
//         }

//         #[test]
//         fn trinary_clamp_outside_range() {
//             let set = empty_set();
//             assert!(approx(
//                 unwrap_f32(
//                     tri(TrinaryOp::Clamp, lit_f32(15.0), lit_f32(0.0), lit_f32(10.0)).eval(&set)
//                 ),
//                 10.0
//             ));
//             assert!(approx(
//                 unwrap_f32(
//                     tri(TrinaryOp::Clamp, lit_f32(-3.0), lit_f32(0.0), lit_f32(10.0)).eval(&set)
//                 ),
//                 0.0
//             ));
//         }

//         #[test]
//         fn trinary_clamp_nan_falls_to_floor() {
//             let set = empty_set();
//             let v = unwrap_f32(
//                 tri(
//                     TrinaryOp::Clamp,
//                     lit_f32(f32::NAN),
//                     lit_f32(0.05),
//                     lit_f32(0.5),
//                 )
//                 .eval(&set),
//             );
//             assert!(approx(v, 0.05));
//         }

//         #[test]
//         fn trinary_clamp_null_value_falls_to_floor() {
//             let set = empty_set();
//             let v = unwrap_f32(
//                 tri(TrinaryOp::Clamp, lit_null(), lit_f32(0.05), lit_f32(0.5)).eval(&set),
//             );
//             assert!(approx(v, 0.05));
//         }

//         // -- ScheduleExpr --------------------------------------------------------

//         #[test]
//         fn schedule_every_through_expr() {
//             let mut e = Expr::Schedule(ScheduleExpr::Every(EveryState::new(3)));
//             let set = empty_set();
//             assert_eq!(e.eval(&set), MetricValue::Bool(false));
//             assert_eq!(e.eval(&set), MetricValue::Bool(false));
//             assert_eq!(e.eval(&set), MetricValue::Bool(true));
//             assert_eq!(e.eval(&set), MetricValue::Bool(false));
//         }

//         // -- SelectExpr ----------------------------------------------------------

//         #[test]
//         fn select_bound_reads_mean() {
//             let mut set = MetricSet::default();
//             let name: SmallStr = SmallStr::from_static("scores");
//             let idx = set.resolve(&name);
//             set.upsert_at(idx, 2.0_f32);
//             set.upsert_at(idx, 4.0_f32);
//             set.upsert_at(idx, 6.0_f32);

//             let mut e = Expr::Select(SelectExpr::Bound(idx, StatField::Mean));
//             assert!(approx(unwrap_f32(e.eval(&set)), 4.0));
//         }

//         #[test]
//         fn select_bound_reads_last_value() {
//             let mut set = MetricSet::default();
//             let idx = set.resolve(&SmallStr::from_static("scores"));
//             set.upsert_at(idx, 1.0_f32);
//             set.upsert_at(idx, 7.0_f32);
//             let mut e = Expr::Select(SelectExpr::Bound(idx, StatField::LastValue));
//             assert!(approx(unwrap_f32(e.eval(&set)), 7.0));
//         }

//         #[test]
//         fn select_bound_count_is_u64() {
//             let mut set = MetricSet::default();
//             let idx = set.resolve(&SmallStr::from_static("c"));
//             set.upsert_at(idx, 1.0_f32);
//             set.upsert_at(idx, 2.0_f32);
//             let mut e = Expr::Select(SelectExpr::Bound(idx, StatField::Count));
//             assert_eq!(e.eval(&set), MetricValue::U64(2));
//         }

//         #[test]
//         fn select_bound_quantile_field_returns_null_until_wired() {
//             // Placeholder behavior — flips when Metric grows quantile rollups.
//             let mut set = MetricSet::default();
//             let idx = set.resolve(&SmallStr::from_static("x"));
//             set.upsert_at(idx, 1.0_f32);
//             let mut e = Expr::Select(SelectExpr::Bound(idx, StatField::Quantile(0.5)));
//             assert_eq!(e.eval(&set), MetricValue::Null);
//         }

//         // -- RollingExpr / compute_rollup ----------------------------------------

//         #[test]
//         fn rolling_mean_over_window() {
//             let set = empty_set();
//             let mut buf = RollingExpr {
//                 child: lit_f32(0.0), // overwritten per eval; we push directly below
//                 rollup: Rollup::Mean,
//                 buffer: WindowBuffer::with_capacity(3),
//             };
//             // Manually push values via the child literal — swap & eval pattern:
//             buf.child = lit_f32(2.0);
//             buf.eval(&set);
//             buf.child = lit_f32(4.0);
//             buf.eval(&set);
//             buf.child = lit_f32(6.0);
//             assert!(approx(unwrap_f32(buf.eval(&set)), 4.0));
//         }

//         #[test]
//         fn rolling_window_drops_oldest() {
//             let set = empty_set();
//             let mut buf = RollingExpr {
//                 child: lit_f32(0.0),
//                 rollup: Rollup::Mean,
//                 buffer: WindowBuffer::with_capacity(2),
//             };
//             buf.child = lit_f32(1.0);
//             buf.eval(&set);
//             buf.child = lit_f32(2.0);
//             buf.eval(&set);
//             buf.child = lit_f32(3.0); // window now [2.0, 3.0]
//             assert!(approx(unwrap_f32(buf.eval(&set)), 2.5));
//         }

//         #[test]
//         fn rolling_reset_clears_buffer() {
//             let set = empty_set();
//             let mut e = rolling(lit_f32(0.0), Rollup::Count, 5);
//             // count goes 1, 2, 3...
//             e.eval(&set);
//             e.eval(&set);
//             assert_eq!(e.eval(&set), MetricValue::U64(3));
//             e.reset();
//             assert_eq!(e.eval(&set), MetricValue::U64(1));
//         }

//         #[test]
//         fn compute_rollup_empty_returns_null_except_count() {
//             assert_eq!(compute_rollup(&[], Rollup::Mean), MetricValue::Null);
//             assert_eq!(compute_rollup(&[], Rollup::Count), MetricValue::U64(0));
//         }

//         #[test]
//         fn compute_rollup_single_element() {
//             assert_eq!(compute_rollup(&[5.0], Rollup::First), MetricValue::F32(5.0));
//             assert_eq!(compute_rollup(&[5.0], Rollup::Last), MetricValue::F32(5.0));
//             assert_eq!(compute_rollup(&[5.0], Rollup::Mean), MetricValue::F32(5.0));
//             assert_eq!(
//                 compute_rollup(&[5.0], Rollup::StdDev),
//                 MetricValue::F32(0.0)
//             );
//             assert_eq!(compute_rollup(&[5.0], Rollup::Count), MetricValue::U64(1));
//         }

//         #[test]
//         fn compute_rollup_first_last() {
//             assert!(approx(
//                 unwrap_f32(compute_rollup(&[1.0, 2.0, 3.0], Rollup::First)),
//                 1.0
//             ));
//             assert!(approx(
//                 unwrap_f32(compute_rollup(&[1.0, 2.0, 3.0], Rollup::Last)),
//                 3.0
//             ));
//         }

//         #[test]
//         fn compute_rollup_sum_mean_min_max() {
//             let vs = [1.0_f32, 2.0, 3.0, 4.0];
//             assert!(approx(unwrap_f32(compute_rollup(&vs, Rollup::Sum)), 10.0));
//             assert!(approx(unwrap_f32(compute_rollup(&vs, Rollup::Mean)), 2.5));
//             assert!(approx(unwrap_f32(compute_rollup(&vs, Rollup::Min)), 1.0));
//             assert!(approx(unwrap_f32(compute_rollup(&vs, Rollup::Max)), 4.0));
//         }

//         #[test]
//         fn compute_rollup_slope_on_line() {
//             let vs: Vec<f32> = (1..=10).map(|i| i as f32).collect();
//             // Perfect ramp — slope should be 1.0
//             assert!(approx(unwrap_f32(compute_rollup(&vs, Rollup::Slope)), 1.0));
//         }

//         #[test]
//         fn compute_rollup_quantile_median() {
//             let vs = [1.0_f32, 2.0, 3.0, 4.0, 5.0];
//             assert!(approx(
//                 unwrap_f32(compute_rollup(&vs, Rollup::Quantile(0.5))),
//                 3.0
//             ));
//         }

//         #[test]
//         fn compute_rollup_quantile_p95() {
//             let vs: Vec<f32> = (1..=100).map(|i| i as f32).collect();
//             // Linear interpolation: q=0.95 of 1..100 → ≈ 95.05
//             let v = unwrap_f32(compute_rollup(&vs, Rollup::Quantile(0.95)));
//             assert!((v - 95.05).abs() < 0.5);
//         }

//         // -- End-to-end ----------------------------------------------------------

//         #[test]
//         fn end_to_end_chained_expr() {
//             // (select(scores).mean() + 1.0) > 5.0
//             let mut set = MetricSet::default();
//             let idx = set.resolve(&SmallStr::from_static("scores"));
//             set.upsert_at(idx, 4.0_f32);
//             set.upsert_at(idx, 6.0_f32); // mean = 5.0

//             let select = Expr::Select(SelectExpr::Bound(idx, StatField::Mean));
//             let added = bin(select, BinaryOp::Add, lit_f32(1.0)); // mean + 1 = 6
//             let mut expr = bin(added, BinaryOp::Gt, lit_f32(5.0)); // 6 > 5 = true

//             assert_eq!(expr.eval(&set), MetricValue::Bool(true));
//         }

//         #[test]
//         fn end_to_end_coalesce_fallback() {
//             // select(missing).last() ?? -1.0  → -1.0
//             let set = MetricSet::default();
//             let select = Expr::Select(SelectExpr::Bound(MetricIdx::INVALID, StatField::LastValue));
//             let mut expr = bin(select, BinaryOp::Coalesce, lit_f32(-1.0));
//             assert!(approx(unwrap_f32(expr.eval(&set)), -1.0));
//         }

//         // -- DSL builder -----------------------------------------------------

//         #[test]
//         fn dsl_literal_via_from() {
//             let mut e: Expr = 3.5_f32.into();
//             let set = empty_set();
//             assert!(approx(unwrap_f32(e.eval(&set)), 3.5));
//         }

//         #[test]
//         fn dsl_select_default_field_is_last_value() {
//             let e = expr::select("scores");
//             match e {
//                 Expr::Select(SelectExpr::Pending(_, StatField::LastValue)) => {}
//                 other => panic!("expected Pending(LastValue), got other variant"),
//             }
//         }

//         #[test]
//         fn dsl_select_mean_sets_stat_field() {
//             let e = expr::select("scores").mean();
//             match e {
//                 Expr::Select(SelectExpr::Pending(_, StatField::Mean)) => {}
//                 other => panic!("expected Pending(Mean), got other variant"),
//             }
//         }

//         #[test]
//         fn dsl_select_quantile_sets_field() {
//             let e = expr::select("scores").quantile(0.95);
//             match e {
//                 Expr::Select(SelectExpr::Pending(_, StatField::Quantile(q))) => {
//                     assert!(approx(q, 0.95));
//                 }
//                 other => panic!("expected Pending(Quantile), got other variant"),
//             }
//         }

//         #[test]
//         fn dsl_rolling_wraps_in_rolling_expr() {
//             let e = expr::select("x").rolling(10);
//             match e {
//                 Expr::Rolling(_) => {}
//                 other => panic!("expected Rolling, got other variant"),
//             }
//         }

//         #[test]
//         fn dsl_rolling_then_mean_sets_rollup() {
//             let e = expr::select("x").rolling(10).mean();
//             match e {
//                 Expr::Rolling(r) => assert_eq!(r.rollup, Rollup::Mean),
//                 other => panic!("expected Rolling(Mean), got other variant"),
//             }
//         }

//         #[test]
//         fn dsl_arithmetic_operator_overloads() {
//             let set = empty_set();
//             // (1.0 + 2.0) * 3.0 - 1.0
//             let mut e = (Expr::from(1.0_f32) + Expr::from(2.0_f32)) * Expr::from(3.0_f32)
//                 - Expr::from(1.0_f32);
//             assert!(approx(unwrap_f32(e.eval(&set)), 8.0));
//         }

//         #[test]
//         fn dsl_comparison_chain() {
//             let set = empty_set();
//             // 5.0 > 3.0 && 5.0 < 10.0
//             let mut e = expr::lit(5.0_f32).gt(3.0).and(expr::lit(5.0_f32).lt(10.0));
//             assert_eq!(e.eval(&set), MetricValue::Bool(true));
//         }

//         #[test]
//         fn dsl_clamp_via_method() {
//             let set = empty_set();
//             let mut e = expr::lit(15.0_f32).clamp(0.0, 10.0);
//             assert!(approx(unwrap_f32(e.eval(&set)), 10.0));
//         }

//         #[test]
//         fn dsl_or_else_falls_back() {
//             let set = empty_set();
//             let mut e = expr::lit(f32::NAN).or_else(7.0);
//             assert!(approx(unwrap_f32(e.eval(&set)), 7.0));
//         }

//         #[test]
//         fn dsl_when_then_otherwise() {
//             let set = empty_set();
//             let mut e = expr::when(true).then(100.0_f32).otherwise(200.0_f32);
//             assert!(approx(unwrap_f32(e.eval(&set)), 100.0));

//             let mut e2 = expr::when(false).then(100.0_f32).otherwise(200.0_f32);
//             assert!(approx(unwrap_f32(e2.eval(&set)), 200.0));
//         }

//         #[test]
//         fn dsl_every_then_otherwise() {
//             let set = empty_set();
//             // Every 2: fires on the 2nd eval.
//             let mut e = expr::every(2).then(1.0_f32).otherwise(0.0_f32);
//             assert!(approx(unwrap_f32(e.eval(&set)), 0.0)); // tick 1: not yet
//             assert!(approx(unwrap_f32(e.eval(&set)), 1.0)); // tick 2: fires
//             assert!(approx(unwrap_f32(e.eval(&set)), 0.0)); // tick 1: reset
//         }

//         #[test]
//         fn dsl_predicates_chain() {
//             let set = empty_set();
//             let mut e = expr::lit(f32::INFINITY).is_finite().not();
//             assert_eq!(e.eval(&set), MetricValue::Bool(true));
//         }

//         #[test]
//         fn dsl_min_max_with() {
//             let set = empty_set();
//             let mut floor = expr::lit(0.5_f32).max_with(0.05);
//             assert!(approx(unwrap_f32(floor.eval(&set)), 0.5));
//             let mut ceil = expr::lit(0.95_f32).min_with(0.5);
//             assert!(approx(unwrap_f32(ceil.eval(&set)), 0.5));
//         }

//         #[test]
//         fn dsl_resolve_pending_to_bound() {
//             let mut set = MetricSet::default();
//             let idx = set.resolve(&SmallStr::from_static("scores"));
//             set.upsert_at(idx, 2.0_f32);
//             set.upsert_at(idx, 4.0_f32);

//             let mut e = expr::select("scores").mean();
//             assert!(e.resolve(&set));

//             match &e {
//                 Expr::Select(SelectExpr::Bound(i, StatField::Mean)) => assert_eq!(*i, idx),
//                 _ => panic!("expected resolved Bound"),
//             }
//             assert!(approx(unwrap_f32(e.eval(&set)), 3.0));
//         }

//         #[test]
//         fn dsl_resolve_returns_false_for_unknown_metric() {
//             let set = MetricSet::default();
//             let mut e = expr::select("does.not.exist").mean();
//             assert!(!e.resolve(&set));
//         }

//         #[test]
//         fn dsl_chained_end_to_end_with_resolve() {
//             // (select("scores").mean() * 2.0).gt(5.0).and(true)
//             let mut set = MetricSet::default();
//             let idx = set.resolve(&SmallStr::from_static("scores"));
//             set.upsert_at(idx, 2.0_f32);
//             set.upsert_at(idx, 4.0_f32); // mean = 3.0

//             let mut e = (expr::select("scores").mean() * Expr::from(2.0_f32))
//                 .gt(5.0)
//                 .and(true);
//             assert!(e.resolve(&set));
//             assert_eq!(e.eval(&set), MetricValue::Bool(true));
//         }

//         #[test]
//         fn dsl_negation_and_not() {
//             let set = empty_set();
//             let mut neg = -expr::lit(3.0_f32);
//             assert!(approx(unwrap_f32(neg.eval(&set)), -3.0));
//             let mut not_true = !expr::lit(true);
//             assert_eq!(not_true.eval(&set), MetricValue::Bool(false));
//         }

//         #[test]
//         fn dsl_rolling_then_quantile_via_method() {
//             // Build via DSL, push 5 values, expect ~median.
//             let set = empty_set();
//             let mut e = expr::lit(1.0_f32).rolling(5).quantile(0.5);
//             // Note: lit(1.0) is constant — pushing it five times gives all-1.0 window
//             for _ in 0..5 {
//                 e.eval(&set);
//             }
//             assert!(approx(unwrap_f32(e.eval(&set)), 1.0));
//         }
//     }
// }
