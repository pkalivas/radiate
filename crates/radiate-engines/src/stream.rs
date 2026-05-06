// //! # Streaming Engine API
// //!
// //! [EngineStream] is a builder-style alternative to [crate::EngineIterator] for driving a
// //! [GeneticEngine]. Instead of yielding generations through the standard [Iterator] trait,
// //! it collects a list of inspectors and stop predicates and applies them inside a single
// //! terminal method ([EngineStream::last], [EngineStream::for_each], etc.).
// //!
// //! Two practical differences from the [Iterator]-based path:
// //!
// //! - **Single materialization.** Terminal methods that don't need ownership
// //!   ([EngineStream::for_each], [EngineStream::count]) never hand the user an owned
// //!   [Generation]. [EngineStream::last] returns exactly one.
// //! - **Composable plan.** Configurators ([EngineStream::until_score], [EngineStream::logging],
// //!   [EngineStream::checkpoint], etc.) just push into the inspector / stop-predicate vectors,
// //!   so the order they're declared in is the order they execute.
// //!
// //! ## Note on cloning
// //!
// //! The current implementation drives the engine via [radiate_core::Engine::next], which
// //! returns an owned [Generation] each step. The per-step clone of the ecosystem still
// //! happens internally — the user-visible API is shaped for a future zero-clone refactor
// //! (splitting `step()` from `next()` on the engine), but the perf win isn't realized yet.

// use crate::{
//     EngineControl, Generation, GenerationView, GeneticEngine, Limit, init_logging,
//     steps::EngineStep,
// };
// use radiate_core::{Chromosome, Engine, EngineStream, Metric, Objective, Optimize, Score};
// use radiate_expr::{AnyValue, Evaluate, Expr};
// use std::collections::VecDeque;
// use std::time::Duration;
// use tracing::info;

// #[cfg(feature = "serde")]
// use crate::{CheckpointWriter, JsonCheckpointWriter};
// #[cfg(feature = "serde")]
// use serde::Serialize;
// #[cfg(feature = "serde")]
// use std::path::{Path, PathBuf};

// type Inspector<E> = Box<dyn Fn(&E)>;
// type StopPredicate<E> = Box<dyn Fn(&E) -> bool>;

// /// A builder-style driver for a [GeneticEngine].
// ///
// /// Configurators (`until_*`, `inspect`, `logging`, `checkpoint`) consume `self`
// /// and return `Self`, accumulating side-effects and termination conditions into
// /// an internal plan. Terminal methods ([Self::last], [Self::for_each],
// /// [Self::count], [Self::collect_owned], [Self::run]) consume the stream and
// /// drive the engine to completion.
// ///
// /// All stop predicates are evaluated as a logical OR — the stream stops when
// /// any one of them returns `true`.
// ///
// /// # Example
// ///
// /// ```rust,ignore
// /// use radiate_engines::*;
// ///
// /// let final_gen = GeneticEngine::builder()
// ///     .codec(FloatCodec::vector(5, 0.0..1.0))
// ///     .fitness_fn(|x: Vec<f32>| x.iter().sum::<f32>())
// ///     .build()
// ///     .stream()
// ///     .logging()
// ///     .until_score(0.01)
// ///     .until_seconds(30.0)
// ///     .last();
// /// ```
// pub struct EngineStreamHandle<E>
// where
//     E: EngineStream,
// {
//     engine: E,
//     inspectors: Vec<Inspector<E::State>>,
//     stops: Vec<StopPredicate<E::State>>,
// }

// impl<E> EngineStreamHandle<E> where E: EngineStream {}

// impl<C, T> EngineStreamHandle<GeneticEngine<C, T>>
// where
//     C: Chromosome + Clone + 'static,
//     T: Clone + Send + Sync + 'static,
// {
//     pub fn new(engine: GeneticEngine<C, T>) -> Self {
//         Self {
//             engine,
//             inspectors: Vec::new(),
//             stops: Vec::new(),
//         }
//     }

//     pub fn last(mut self) -> Option<Generation<C, T>> {
//         let last_gen: Option<Generation<C, T>>;
//         loop {
//             if let Err(e) = self.engine.step() {
//                 panic!("{e}");
//             }

//             let context = self.engine.state();
//             for ins in &mut self.inspectors {
//                 ins(context);
//             }

//             let should_stop = self.stops.iter_mut().any(|p| p(context));
//             if should_stop {
//                 last_gen = Some(Generation::from(context));
//                 break;
//             }
//         }

//         return last_gen;
//     }

//     pub fn until_score(mut self, target: impl Into<Score>) -> Self {
//         let target = target.into();
//         self.stops.push(Box::new(move |g| match &g.objective {
//             Objective::Single(Optimize::Minimize) => g
//                 .score
//                 .as_ref()
//                 .map(|score| score <= &target)
//                 .unwrap_or(false),
//             Objective::Single(Optimize::Maximize) => g
//                 .score
//                 .as_ref()
//                 .map(|score| score >= &target)
//                 .unwrap_or(false),
//             Objective::Multi(objs) => g
//                 .score
//                 .as_ref()
//                 .map(|score| {
//                     score.iter().enumerate().all(|(i, s)| match objs[i] {
//                         Optimize::Minimize => s <= &target[i],
//                         Optimize::Maximize => s >= &target[i],
//                     })
//                 })
//                 .unwrap_or(false),
//         }));
//         self
//     }
// }
