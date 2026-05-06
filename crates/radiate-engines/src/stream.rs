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

// use crate::{EngineControl, Generation, GeneticEngine, Limit, init_logging};
// use radiate_core::{Chromosome, Engine, Metric, Objective, Optimize, Score};
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

// type Inspector<C, T> = Box<dyn FnMut(&Generation<C, T>)>;
// type StopPredicate<C, T> = Box<dyn FnMut(&Generation<C, T>) -> bool>;

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
// pub struct EngineStream<C, T>
// where
//     C: Chromosome + Clone,
//     T: Clone + Send + Sync + 'static,
// {
//     engine: GeneticEngine<C, T>,
//     control: Option<EngineControl>,
//     inspectors: Vec<Inspector<C, T>>,
//     stops: Vec<StopPredicate<C, T>>,
// }

// impl<C, T> EngineStream<C, T>
// where
//     C: Chromosome + Clone,
//     T: Clone + Send + Sync + 'static,
// {
//     /* =========================== Configurators =========================== */
//     /// Pushes a side-effect closure that runs on every yielded generation.
//     pub fn inspect(mut self, f: impl FnMut(&Generation<C, T>) + 'static) -> Self {
//         self.inspectors.push(Box::new(f));
//         self
//     }

//     /// Pushes a stop predicate. The stream stops when the predicate returns `true`.
//     pub fn until(mut self, f: impl FnMut(&Generation<C, T>) -> bool + 'static) -> Self {
//         self.stops.push(Box::new(f));
//         self
//     }

//     /// Stop when the generation index reaches `max`.
//     pub fn until_index(self, max: usize) -> Self {
//         self.until(move |g| g.index() >= max)
//     }

//     /// Stop after `secs` of cumulative engine time (per the metric system, not wall clock).
//     pub fn until_seconds(self, secs: f64) -> Self {
//         let limit = Duration::from_secs_f64(secs);
//         self.until(move |g| g.time() >= limit)
//     }

//     /// Stop after the given duration of cumulative engine time.
//     pub fn until_duration(self, limit: impl Into<Duration>) -> Self {
//         let limit = limit.into();
//         self.until(move |g| g.time() >= limit)
//     }

//     /// Stop when the best score reaches the target, respecting the optimization
//     /// direction. Multi-objective requires every objective to meet its target.
//     pub fn until_score(self, target: impl Into<Score>) -> Self {
//         let target = target.into();
//         self.until(move |g| match g.objective() {
//             Objective::Single(Optimize::Minimize) => g.score() <= &target,
//             Objective::Single(Optimize::Maximize) => g.score() >= &target,
//             Objective::Multi(objs) => g.score().iter().enumerate().all(|(i, s)| match objs[i] {
//                 Optimize::Minimize => s <= &target[i],
//                 Optimize::Maximize => s >= &target[i],
//             }),
//         })
//     }

//     /// Stop when the spread of the last `window` scores falls below `epsilon`.
//     pub fn until_converged(self, window: usize, epsilon: f32) -> Self {
//         assert!(window > 0, "Window size must be greater than 0");
//         assert!(epsilon >= 0.0, "Epsilon must be non-negative");
//         let mut history: VecDeque<f32> = VecDeque::with_capacity(window);
//         self.until(move |g| {
//             history.push_back(g.score().as_f32());
//             if history.len() > window {
//                 history.pop_front();
//             }
//             if history.len() == window {
//                 let first = *history.front().unwrap();
//                 let last = *history.back().unwrap();
//                 (first - last).abs() < epsilon
//             } else {
//                 false
//             }
//         })
//     }

//     /// Stop after `patience` generations with no improvement above `min_improvement`.
//     /// Direction is taken from the engine's [Objective] — minimize and maximize are
//     /// both handled correctly.
//     pub fn until_stagnant(self, patience: usize, min_improvement: f32) -> Self {
//         assert!(patience > 0, "Patience must be greater than 0");
//         assert!(
//             min_improvement >= 0.0,
//             "Min improvement must be non-negative"
//         );
//         let mut best: Option<f32> = None;
//         let mut count: usize = 0;
//         self.until(move |g| {
//             let current = g.score().as_f32();
//             match best {
//                 None => {
//                     best = Some(current);
//                 }
//                 Some(b) => {
//                     let improved = match g.objective() {
//                         Objective::Single(Optimize::Maximize) => current - b > min_improvement,
//                         Objective::Single(Optimize::Minimize) => b - current > min_improvement,
//                         Objective::Multi(_) => (current - b).abs() > min_improvement,
//                     };
//                     if improved {
//                         best = Some(current);
//                         count = 0;
//                     } else {
//                         count += 1;
//                     }
//                 }
//             }
//             count >= patience
//         })
//     }

//     /// Stop when a named metric satisfies the predicate. Missing metric is treated
//     /// as "predicate not satisfied" rather than a panic.
//     pub fn until_metric(self, name: &str, predicate: impl Fn(&Metric) -> bool + 'static) -> Self {
//         let name = name.to_string();
//         self.until(move |g| {
//             g.metrics()
//                 .get(&name)
//                 .map(|m| predicate(m))
//                 .unwrap_or(false)
//         })
//     }

//     /// Stop when the expression evaluates to `Bool(true)` against the metrics.
//     pub fn until_expr(self, mut expr: Expr) -> Self {
//         self.until(move |g| {
//             matches!(
//                 expr.eval(g.metrics()).unwrap_or(AnyValue::Null),
//                 AnyValue::Bool(true)
//             )
//         })
//     }

//     /// Apply a [Limit] (or any [Into<Limit>]). Combined limits flatten recursively.
//     pub fn limit(self, limit: impl Into<Limit>) -> Self {
//         match limit.into() {
//             Limit::Generation(n) => self.until_index(n),
//             Limit::Seconds(d) => self.until_duration(d),
//             Limit::Score(s) => self.until_score(s),
//             Limit::Convergence(w, e) => self.until_converged(w, e),
//             Limit::Metric(name, pred) => {
//                 self.until(move |g| g.metrics().get(&name).map(|m| pred(m)).unwrap_or(false))
//             }
//             Limit::Expr(expr) => self.until_expr(expr),
//             Limit::Combined(limits) => limits.into_iter().fold(self, |s, l| s.limit(l)),
//         }
//     }

//     /// Log every generation through `tracing`.
//     pub fn logging(self) -> Self {
//         self.log_every(1)
//     }

//     /// Log every `interval`-th generation.
//     pub fn log_every(self, interval: usize) -> Self {
//         init_logging();
//         self.inspect(move |g| {
//             if !g.index().is_multiple_of(interval) {
//                 return;
//             }
//             match g.objective() {
//                 Objective::Single(_) => info!(
//                     "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
//                     g.index(),
//                     g.score().as_f32(),
//                     g.time()
//                 ),
//                 Objective::Multi(_) => {
//                     let v = g
//                         .metrics()
//                         .front_size()
//                         .map(|e| e.last_value())
//                         .unwrap_or(0.0);
//                     info!(
//                         "Epoch {:<4} | Front Size: {:.3} | Time: {:>5.2?}",
//                         g.index(),
//                         v,
//                         g.time()
//                     );
//                 }
//             }
//         })
//     }

//     /// Write a JSON checkpoint every `interval` generations.
//     #[cfg(feature = "serde")]
//     pub fn checkpoint(self, interval: usize, folder_path: impl AsRef<Path>) -> Self
//     where
//         C: Serialize + 'static,
//         T: Serialize,
//     {
//         self.checkpoint_with(interval, folder_path, Box::new(JsonCheckpointWriter))
//     }

//     /// Write a checkpoint every `interval` generations using a custom writer.
//     #[cfg(feature = "serde")]
//     pub fn checkpoint_with(
//         self,
//         interval: usize,
//         folder_path: impl AsRef<Path>,
//         mut writer: Box<dyn CheckpointWriter<C, T>>,
//     ) -> Self
//     where
//         C: Serialize + 'static,
//         T: Serialize,
//     {
//         let path = PathBuf::from(folder_path.as_ref());
//         self.inspect(move |g| {
//             if !g.index().is_multiple_of(interval) {
//                 return;
//             }
//             if !path.exists() {
//                 if let Err(e) = std::fs::create_dir_all(&path) {
//                     eprintln!("Failed to create checkpoint directory: {e}");
//                     return;
//                 }
//             }
//             let file = path.join(format!("chckpnt_{}.{}", g.index(), writer.extension()));
//             if let Err(e) = writer.write_checkpoint(file, g) {
//                 eprintln!("Failed to write checkpoint: {e}");
//             }
//         })
//     }

//     /* ============================ Terminals ============================ */
//     /// Drive the engine until any stop predicate fires (or the control says stop).
//     /// Returns the final generation as the only owned [Generation] handed back.
//     pub fn last(mut self) -> Option<Generation<C, T>> {
//         let mut last_gen: Option<Generation<C, T>> = None;
//         loop {
//             if self.is_stopped() {
//                 break;
//             }
//             let epoch = match self.engine.next() {
//                 Ok(g) => g,
//                 Err(e) => panic!("{e}"),
//             };
//             for ins in &mut self.inspectors {
//                 ins(&epoch);
//             }
//             let stop = self.stops.iter_mut().any(|p| p(&epoch));
//             last_gen = Some(epoch);
//             if stop {
//                 break;
//             }
//         }
//         last_gen
//     }

//     /// Alias for [Self::last].
//     pub fn run(self) -> Option<Generation<C, T>> {
//         self.last()
//     }

//     /// Drive the engine, calling `f` on each generation. Never returns ownership.
//     pub fn for_each(mut self, mut f: impl FnMut(&Generation<C, T>)) {
//         loop {
//             if self.is_stopped() {
//                 break;
//             }
//             let epoch = match self.engine.next() {
//                 Ok(g) => g,
//                 Err(e) => panic!("{e}"),
//             };
//             for ins in &mut self.inspectors {
//                 ins(&epoch);
//             }
//             f(&epoch);
//             if self.stops.iter_mut().any(|p| p(&epoch)) {
//                 break;
//             }
//         }
//     }

//     /// Drive the engine and return how many generations ran.
//     pub fn count(mut self) -> usize {
//         let mut n: usize = 0;
//         loop {
//             if self.is_stopped() {
//                 break;
//             }
//             let epoch = match self.engine.next() {
//                 Ok(g) => g,
//                 Err(e) => panic!("{e}"),
//             };
//             n += 1;
//             for ins in &mut self.inspectors {
//                 ins(&epoch);
//             }
//             if self.stops.iter_mut().any(|p| p(&epoch)) {
//                 break;
//             }
//         }
//         n
//     }

//     /// Drive the engine and collect every generation into a Vec. Pays a clone per step.
//     pub fn collect_owned(mut self) -> Vec<Generation<C, T>> {
//         let mut out: Vec<Generation<C, T>> = Vec::new();
//         loop {
//             if self.is_stopped() {
//                 break;
//             }
//             let epoch = match self.engine.next() {
//                 Ok(g) => g,
//                 Err(e) => panic!("{e}"),
//             };
//             for ins in &mut self.inspectors {
//                 ins(&epoch);
//             }
//             let stop = self.stops.iter_mut().any(|p| p(&epoch));
//             out.push(epoch);
//             if stop {
//                 break;
//             }
//         }
//         out
//     }

//     fn is_stopped(&self) -> bool {
//         self.control.as_ref().is_some_and(|c| c.is_stopped())
//     }
// }

// /// Extension trait that adds [Self::stream] to [GeneticEngine] without modifying
// /// the engine module. Calling `stream()` consumes the engine and auto-creates an
// /// [EngineControl] so external stop signals work out of the box (the
// /// [crate::EngineIterator] path captures control only if one was created earlier).
// pub trait EngineStreamExt<C, T>: Sized
// where
//     C: Chromosome + Clone,
//     T: Clone + Send + Sync + 'static,
// {
//     fn stream(self) -> EngineStream<C, T>;
// }

// impl<C, T> EngineStreamExt<C, T> for GeneticEngine<C, T>
// where
//     C: Chromosome + Clone,
//     T: Clone + Send + Sync + 'static,
// {
//     fn stream(mut self) -> EngineStream<C, T> {
//         let control = Some(self.control());
//         EngineStream {
//             engine: self,
//             control,
//             inspectors: Vec::new(),
//             stops: Vec::new(),
//         }
//     }
// }
