pub mod alterers;
pub mod codexes;
pub mod context;
pub mod domain;
pub mod engine;
pub mod genome;
pub mod objectives;
pub mod params;
pub mod selectors;
pub mod stats;

pub use alterers::*;
pub use codexes::{
    BitCodex, CharCodex, Codex, FloatCodex, FnCodex, IntCodex, PermutationCodex, SubSetCodex,
};
pub use context::*;
pub use domain::*;
pub use engine::*;
pub use genome::*;
pub use objectives::*;
pub use params::*;
pub use selectors::*;
pub use stats::*;

pub trait EngineCompoment {
    fn name(&self) -> &'static str;
}
