// use super::objectives::Score;
// use super::{MetricSet, Phenotype};
// use crate::Chromosome;
// use crate::objectives::Front;
// use radiate_core::{Ecosystem, Objective, Problem};
// use std::sync::Arc;

// /// The context of the genetic engine. This struct contains the current state of the genetic engine
// /// at any given time. This includes:
// /// * current population
// /// * current best individual
// /// * current index - the number of generations that have passed
// /// * timer - the duration of time the engine has been running
// /// * metrics - a set of metrics that are collected during the run
// /// * current best score - the score of the current best individual
// /// * front - the current pareto front of the population (if multi-objective)
// ///
// /// The EngineContext is passed to the user-defined closure that is executed each generation. The user
// /// can use the EngineContext to access the current state of the genetic engine and make decisions based
// /// on the current state on how to proceed.
// ///
// /// # Type Parameters
// /// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
// /// - `T`: The type of the best individual in the population.
// pub struct EngineContext<C, T>
// where
//     C: Chromosome,
// {
//     pub ecosystem: Ecosystem<C>,
//     pub best: T,
//     pub index: usize,
//     pub metrics: MetricSet,
//     pub score: Option<Score>,
//     pub front: Front<Phenotype<C>>,
//     pub objective: Objective,
//     pub problem: Arc<dyn Problem<C, T>>,
// }

// impl<C, T> Clone for EngineContext<C, T>
// where
//     C: Chromosome,
//     T: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             ecosystem: self.ecosystem.clone(),
//             best: self.best.clone(),
//             index: self.index,
//             metrics: self.metrics.clone(),
//             score: self.score.clone(),
//             front: self.front.clone(),
//             objective: self.objective.clone(),
//             problem: Arc::clone(&self.problem),
//         }
//     }
// }
