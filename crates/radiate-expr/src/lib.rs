mod builder;
mod compile;
mod expr;
mod from;
pub mod nodes;
mod set;
mod traits;

pub use expr::{Expr, ExprNode};
pub use nodes::select::Selector;
pub use set::ExprSet;
pub(crate) use traits::ExprResult;
pub use traits::{EvalExpr, EvalNoInput, ExprSelect};

pub mod metric_fields {
    use radiate_utils::SmallStr;

    pub const LAST_VALUE: SmallStr = SmallStr::from_static("last_value");
    pub const COUNT: SmallStr = SmallStr::from_static("count");
    pub const MEAN: SmallStr = SmallStr::from_static("mean");
    pub const VARIANCE: SmallStr = SmallStr::from_static("variance");
    pub const STDDEV: SmallStr = SmallStr::from_static("stddev");
    pub const SKEWNESS: SmallStr = SmallStr::from_static("skewness");
    pub const KURTOSIS: SmallStr = SmallStr::from_static("kurtosis");
    pub const MIN: SmallStr = SmallStr::from_static("min");
    pub const MAX: SmallStr = SmallStr::from_static("max");
    pub const SUM: SmallStr = SmallStr::from_static("sum");
    pub const GENERATION: SmallStr = SmallStr::from_static("generation");
    pub const UPDATE_COUNT: SmallStr = SmallStr::from_static("update_count");
}
