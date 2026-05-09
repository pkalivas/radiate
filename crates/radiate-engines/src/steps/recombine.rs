use crate::steps::EngineStep;
use radiate_core::{
    Alterer, Chromosome, Ecosystem, Lineage, MetricSet, Objective, Optimize, Population, Score,
    Select,
};
use radiate_error::Result;
use radiate_utils::VersionedCounts;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SurvivorConfig<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) selector: Arc<dyn Select<C>>,
    pub(crate) names: (&'static str, &'static str),
}

#[derive(Clone)]
pub struct OffspringConfig<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) selector: Arc<dyn Select<C>>,
    pub(crate) alters: Vec<Alterer<C>>,
    pub(crate) names: (&'static str, &'static str),
}

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor: SurvivorConfig<C>,
    pub(crate) offspring: OffspringConfig<C>,
    pub(crate) objective: Objective,
    pub(crate) lineage: Arc<RwLock<Lineage>>,
    pub(crate) survivor_counts: VersionedCounts,
    pub(crate) offspring_counts: VersionedCounts,
}

impl<C> EngineStep<C> for RecombineStep<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        if ecosystem.species().is_some() {
            self.create_with_species(generation, ecosystem, metrics);
        } else {
            self.combined_create(generation, ecosystem, metrics);
        }
        Ok(())
    }
}

impl<C> RecombineStep<C>
where
    C: Chromosome + PartialEq + Clone,
{
    /// Non-species path: one descending walk that builds both survivors and
    /// offspring from the population. Each unique source idx yields exactly
    /// one `swap_remove` move regardless of how many survivor or offspring
    /// slots it fills, so we save a clone per idx that appears in both
    /// selections (or in survivors only).
    fn combined_create(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) {
        let mut lineage = self.lineage.write().unwrap();
        let pop_len = ecosystem.population().len();

        // Phase 1: run both selectors on the unmutated population.
        let s_timer = std::time::Instant::now();
        let s_indices = self.survivor.selector.select(
            ecosystem.population(),
            &self.objective,
            self.survivor.count,
        );
        let s_elapsed = s_timer.elapsed();

        let o_timer = std::time::Instant::now();
        let o_indices = self.offspring.selector.select(
            ecosystem.population(),
            &self.objective,
            self.offspring.count,
        );
        let o_elapsed = o_timer.elapsed();

        // Phase 2: aggregate per-idx counts into the two reusable buffers.
        self.offspring_counts.begin(pop_len);
        for &idx in &o_indices {
            self.offspring_counts.bump(idx);
        }

        self.survivor_counts.begin(pop_len);
        for &idx in &s_indices {
            self.survivor_counts.bump(idx);
        }

        // Phase 3: single descending walk over the union of selected indices.
        // For each unique source idx with total = s + o > 0, emit (total - 1)
        // clones distributed to whichever bucket still needs entries, then
        // swap_remove the last one and place it in whichever bucket has room.
        let mut survivors = Population::with_capacity(self.survivor.count);
        let mut offspring = Population::with_capacity(self.offspring.count);

        // The iterator borrows the count buffers immutably; `pop` borrows
        // `ecosystem` mutably. Distinct objects, so the borrows coexist.
        let iter = self
            .survivor_counts
            .iter_pair_live_rev(&self.offspring_counts);
        let pop = ecosystem.population_mut();

        for (idx, s, o) in iter {
            let (mut s_left, mut o_left) = (s as usize, o as usize);
            let total = s_left + o_left;

            for _ in 0..total - 1 {
                if s_left > 0 {
                    survivors.push(pop[idx].clone());
                    s_left -= 1;
                } else {
                    offspring.push(pop[idx].clone());
                    o_left -= 1;
                }
            }

            let moved = pop.swap_remove(idx);
            if s_left > 0 {
                survivors.push(moved);
            } else {
                let _ = o_left;
                offspring.push(moved);
            }
        }

        metrics.upsert((self.survivor.names.0, survivors.len()));
        metrics.upsert((self.survivor.names.1, s_elapsed));
        metrics.upsert((self.offspring.names.0, offspring.len()));
        metrics.upsert((self.offspring.names.1, o_elapsed));

        // Phase 4: alter offspring only. Survivors pass through unchanged.
        self.objective.sort(&mut offspring);
        for alt in &mut self.offspring.alters {
            alt.alter(&mut offspring, &mut lineage, metrics, generation);
        }

        let pop = ecosystem.population_mut();
        pop.clear();
        pop.extend(survivors);
        pop.extend(offspring);
    }

    /// Species path: per-species reproduction. Survivors are selected globally
    /// via the survivor selector, then per-species offspring quotas drive
    /// scoped selection + alteration within each species' drained sub-pop.
    fn create_with_species(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) {
        let s_timer = std::time::Instant::now();
        let s_indices = self.survivor.selector.select(
            ecosystem.population(),
            &self.objective,
            self.survivor.count,
        );
        metrics.upsert((self.survivor.names.0, s_indices.len()));
        metrics.upsert((self.survivor.names.1, s_timer.elapsed()));
        let survivors: Population<C> = s_indices
            .into_iter()
            .map(|p| ecosystem.population()[p].clone())
            .collect();

        let mut lineage = self.lineage.write().unwrap();
        let (species, population) = ecosystem.species_population_mut();
        let species = species.expect("species present");

        let mut species_scores = species
            .iter()
            .filter_map(|spec| spec.score())
            .collect::<Vec<_>>();

        if let Objective::Single(Optimize::Minimize) = &self.objective {
            species_scores.reverse();
        }

        let quotas = self.quotas_from_scores(&species_scores);

        let mut next_population = Population::with_capacity(self.offspring.count);
        for (species, count) in species.iter().zip(quotas.iter()) {
            let mut pop = population
                .drain_species(species.id())
                .collect::<Population<C>>();

            self.objective.sort(&mut pop);

            let time = std::time::Instant::now();
            let mut offspring = self
                .offspring
                .selector
                .select(&pop, &self.objective, *count)
                .into_iter()
                .map(|p| pop[p].clone())
                .collect::<Population<C>>();

            metrics.upsert((self.offspring.names.0, offspring.len()));
            metrics.upsert((self.offspring.names.1, time.elapsed()));

            self.objective.sort(&mut offspring);

            for alt in &mut self.offspring.alters {
                alt.alter(&mut offspring, &mut lineage, metrics, generation);
            }

            next_population.extend(offspring);
        }

        drop(lineage);
        let pop = ecosystem.population_mut();
        pop.clear();
        pop.extend(survivors);
        pop.extend(next_population);
    }

    #[inline]
    fn quotas_from_scores(&self, scores: &[&Score]) -> Vec<usize> {
        let n = scores.len();
        if n == 0 || self.offspring.count == 0 {
            return vec![0; n];
        }

        let raw_scores = scores.iter().map(|s| s.as_f32()).collect::<Vec<f32>>();
        let mut min_score = raw_scores.iter().cloned().fold(f32::INFINITY, f32::min);
        if !min_score.is_finite() {
            min_score = 0.0;
        }

        let shifted = raw_scores
            .iter()
            .map(|s| (s - min_score).max(0.0))
            .collect::<Vec<f32>>();

        let sum = shifted.iter().sum::<f32>();

        if sum <= f32::EPSILON {
            let base = self.offspring.count / n;
            let mut quotas = vec![base; n];
            let mut remaining = self.offspring.count - base * n;
            let mut i = 0;
            while remaining > 0 {
                quotas[i] += 1;
                remaining -= 1;
                i += 1;
            }

            return quotas;
        }

        let total = self.offspring.count as f32;

        let mut quotas = Vec::with_capacity(n);
        let mut fracs = Vec::with_capacity(n);
        let mut assigned = 0;

        for (idx, w) in shifted.iter().enumerate() {
            let p = *w / sum;
            let exact = p * total;
            let base = exact.floor() as usize;
            let frac = exact - base as f32;

            quotas.push(base);
            fracs.push((frac, idx));
            assigned += base;
        }

        let remaining = self.offspring.count.saturating_sub(assigned);
        fracs.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, idx) in fracs.iter().take(remaining) {
            quotas[*idx] += 1;
        }

        quotas
    }
}

// use crate::steps::EngineStep;
// use radiate_core::{
//     Alterer, Chromosome, Ecosystem, Lineage, MetricSet, Objective, Optimize, Population, Score,
//     Select, Species,
// };
// use radiate_error::Result;
// use radiate_utils::VersionedCounts;
// use std::sync::{Arc, RwLock};

// enum Operator {
//     Offspring(usize),
//     Survivor(usize),
// }

// pub struct RecombineStep<C: Chromosome> {
//     pub(crate) survivor_handle: SurvivorRecombineHandle<C>,
//     pub(crate) offspring_handle: OffspringRecombineHandle<C>,
// }

// impl<C> EngineStep<C> for RecombineStep<C>
// where
//     C: Chromosome + PartialEq + Clone,
// {
//     #[inline]
//     fn execute(
//         &mut self,
//         generation: usize,
//         ecosystem: &mut Ecosystem<C>,
//         metrics: &mut MetricSet,
//     ) -> Result<()> {
//         let survivors = self.survivor_handle.select(ecosystem, metrics);

//         let (species, population) = ecosystem.species_population_mut();

//         let offspring = if let Some(species) = species {
//             self.offspring_handle
//                 .create_with_species(generation, species, population, metrics)
//         } else {
//             self.offspring_handle.create(generation, ecosystem, metrics)
//         };

//         let population = ecosystem.population_mut();

//         population.clear();
//         population.extend(survivors);
//         population.extend(offspring);

//         Ok(())
//     }
// }

// #[derive(Clone)]
// pub struct SurvivorRecombineHandle<C: Chromosome> {
//     pub(crate) count: usize,
//     pub(crate) objective: Objective,
//     pub(crate) selector: Arc<dyn Select<C>>,
//     pub(crate) names: (&'static str, &'static str),
// }

// impl<C> SurvivorRecombineHandle<C>
// where
//     C: Chromosome + Clone,
// {
//     #[inline]
//     pub fn select(&self, ecosystem: &Ecosystem<C>, metrics: &mut MetricSet) -> Population<C> {
//         let time = std::time::Instant::now();
//         let survivors = self
//             .selector
//             .select(ecosystem.population(), &self.objective, self.count);
//         metrics.upsert((self.names.0, survivors.len()));
//         metrics.upsert((self.names.1, time.elapsed()));
//         survivors
//             .into_iter()
//             .map(|p| ecosystem.population()[p].clone())
//             .collect()
//     }
// }

// #[derive(Clone)]
// pub struct OffspringRecombineHandle<C: Chromosome> {
//     pub(crate) count: usize,
//     pub(crate) objective: Objective,
//     pub(crate) selector: Arc<dyn Select<C>>,
//     pub(crate) alters: Vec<Alterer<C>>,
//     pub(crate) lineage: Arc<RwLock<Lineage>>,
//     pub(crate) names: (&'static str, &'static str),
//     pub(crate) offspring_counts: VersionedCounts,
//     pub(crate) survivor_counts: VersionedCounts,
// }

// impl<C> OffspringRecombineHandle<C>
// where
//     C: Chromosome + PartialEq + Clone,
// {
//     #[inline]
//     pub fn create_with_species(
//         &mut self,
//         generation: usize,
//         species: &[Species<C>],
//         population: &mut Population<C>,
//         metrics: &mut MetricSet,
//     ) -> Population<C> {
//         let mut lineage = self.lineage.write().unwrap();

//         let mut species_scores = species
//             .iter()
//             .filter_map(|spec| spec.score())
//             .collect::<Vec<_>>();

//         if let Objective::Single(Optimize::Minimize) = &self.objective {
//             species_scores.reverse();
//         }

//         let quotas = self.quotas_from_scores(&species_scores);

//         let mut next_population = Population::with_capacity(self.count);
//         for (species, count) in species.iter().zip(quotas.iter()) {
//             let mut pop = population
//                 .drain_species(species.id())
//                 .collect::<Population<C>>();

//             self.objective.sort(&mut pop);

//             let time = std::time::Instant::now();

//             let mut offspring = self
//                 .selector
//                 .select(&pop, &self.objective, *count)
//                 .into_iter()
//                 .map(|p| pop[p].clone())
//                 .collect::<Population<C>>();

//             metrics.upsert((self.names.0, offspring.len()));
//             metrics.upsert((self.names.1, time.elapsed()));

//             self.objective.sort(&mut offspring);

//             self.alters.iter_mut().for_each(|alt| {
//                 alt.alter(&mut offspring, &mut lineage, metrics, generation);
//             });

//             next_population.extend(offspring);
//         }

//         next_population
//     }

//     #[inline]
//     pub fn create(
//         &mut self,
//         generation: usize,
//         ecosystem: &mut Ecosystem<C>,
//         metrics: &mut MetricSet,
//     ) -> Population<C> {
//         let mut lineage = self.lineage.write().unwrap();

//         let timer = std::time::Instant::now();

//         let pop_len = ecosystem.population().len();
//         let indicies = self
//             .selector
//             .select(ecosystem.population(), &self.objective, self.count);

//         self.offspring_counts.begin(pop_len);
//         for &idx in indicies.iter() {
//             self.offspring_counts.bump(idx);
//         }

//         let mut offspring = Population::with_capacity(self.count);
//         let pop = ecosystem.population_mut();

//         for (idx, k) in self.offspring_counts.iter_live_rev() {
//             for _ in 0..k - 1 {
//                 offspring.push(pop[idx].clone());
//             }
//             offspring.push(pop.swap_remove(idx));
//         }

//         metrics.upsert((self.names.0, offspring.len()));
//         metrics.upsert((self.names.1, timer.elapsed()));

//         self.objective.sort(&mut offspring);

//         self.alters.iter_mut().for_each(|alt| {
//             alt.alter(&mut offspring, &mut lineage, metrics, generation);
//         });

//         offspring
//     }

//     pub fn select(
//         &self,
//         ecosystem: &Ecosystem<C>,
//         selector: Arc<dyn Select<C>>,
//         count: usize,
//     ) -> Population<C> {
//         let offspring = self
//             .selector
//             .select(ecosystem.population(), &self.objective, self.count);
//         offspring
//             .into_iter()
//             .map(|p| ecosystem.population()[p].clone())
//             .collect()
//     }

//     #[inline]
//     fn quotas_from_scores(&self, scores: &[&Score]) -> Vec<usize> {
//         let n = scores.len();
//         if n == 0 || self.count == 0 {
//             return vec![0; n];
//         }

//         let raw_scores = scores.iter().map(|s| s.as_f32()).collect::<Vec<f32>>();
//         let mut min_score = raw_scores.iter().cloned().fold(f32::INFINITY, f32::min);
//         if !min_score.is_finite() {
//             min_score = 0.0;
//         }

//         let shifted = raw_scores
//             .iter()
//             .map(|s| (s - min_score).max(0.0))
//             .collect::<Vec<f32>>();

//         let sum = shifted.iter().sum::<f32>();

//         if sum <= f32::EPSILON {
//             let base = self.count / n;
//             let mut quotas = vec![base; n];
//             let mut remaining = self.count - base * n;
//             let mut i = 0;
//             while remaining > 0 {
//                 quotas[i] += 1;
//                 remaining -= 1;
//                 i += 1;
//             }

//             return quotas;
//         }

//         let total = self.count as f32;

//         let mut quotas = Vec::with_capacity(n);
//         let mut fracs = Vec::with_capacity(n);
//         let mut assigned = 0;

//         for (idx, w) in shifted.iter().enumerate() {
//             let p = *w / sum;
//             let exact = p * total;
//             let base = exact.floor() as usize;
//             let frac = exact - base as f32;

//             quotas.push(base);
//             fracs.push((frac, idx));
//             assigned += base;
//         }

//         let remaining = self.count.saturating_sub(assigned);
//         fracs.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

//         for (_, idx) in fracs.iter().take(remaining) {
//             quotas[*idx] += 1;
//         }

//         quotas
//     }
// }

// // use crate::steps::EngineStep;
// // use radiate_core::{
// //     Alterer, Chromosome, Ecosystem, Lineage, MetricSet, Objective, Optimize, Population, Score,
// //     Select, Species,
// // };
// // use radiate_error::Result;
// // use radiate_utils::VersionedCounts;
// // use std::sync::{Arc, RwLock};

// // pub struct RecombineStep<C: Chromosome> {
// //     pub(crate) survivor_handle: SurvivorRecombineHandle<C>,
// //     pub(crate) offspring_handle: OffspringRecombineHandle<C>,
// // }

// // impl<C> EngineStep<C> for RecombineStep<C>
// // where
// //     C: Chromosome + PartialEq + Clone,
// // {
// //     #[inline]
// //     fn execute(
// //         &mut self,
// //         generation: usize,
// //         ecosystem: &mut Ecosystem<C>,
// //         metrics: &mut MetricSet,
// //     ) -> Result<()> {
// //         // Species path is unchanged: per-species reproduction still uses the
// //         // legacy survivor.select + offspring.create_with_species coordination.
// //         if ecosystem.species().is_some() {
// //             let survivors = self.survivor_handle.select(ecosystem, metrics);
// //             let (species, population) = ecosystem.species_population_mut();
// //             let offspring = self.offspring_handle.create_with_species(
// //                 generation,
// //                 species.expect("species present"),
// //                 population,
// //                 metrics,
// //             );

// //             let pop = ecosystem.population_mut();
// //             pop.clear();
// //             pop.extend(survivors);
// //             pop.extend(offspring);
// //             return Ok(());
// //         }

// //         // Non-species path: combined survivor + offspring selection in one
// //         // descending walk over pop. Each unique source idx yields exactly one
// //         // swap_remove move regardless of how many survivor or offspring slots
// //         // it fills, so we save a clone per idx that appears in BOTH selections
// //         // (or in survivors only).
// //         self.combined_create(generation, ecosystem, metrics);
// //         Ok(())
// //     }
// // }

// // #[derive(Clone)]
// // pub struct SurvivorRecombineHandle<C: Chromosome> {
// //     pub(crate) count: usize,
// //     pub(crate) objective: Objective,
// //     pub(crate) selector: Arc<dyn Select<C>>,
// //     pub(crate) names: (&'static str, &'static str),
// // }

// // impl<C> SurvivorRecombineHandle<C>
// // where
// //     C: Chromosome + Clone,
// // {
// //     #[inline]
// //     pub fn select(&self, ecosystem: &Ecosystem<C>, metrics: &mut MetricSet) -> Population<C> {
// //         let time = std::time::Instant::now();
// //         let survivors = self
// //             .selector
// //             .select(ecosystem.population(), &self.objective, self.count);
// //         metrics.upsert((self.names.0, survivors.len()));
// //         metrics.upsert((self.names.1, time.elapsed()));
// //         survivors
// //             .into_iter()
// //             .map(|p| ecosystem.population()[p].clone())
// //             .collect()
// //     }
// // }

// // #[derive(Clone)]
// // pub struct OffspringRecombineHandle<C: Chromosome> {
// //     pub(crate) count: usize,
// //     pub(crate) objective: Objective,
// //     pub(crate) selector: Arc<dyn Select<C>>,
// //     pub(crate) alters: Vec<Alterer<C>>,
// //     pub(crate) lineage: Arc<RwLock<Lineage>>,
// //     pub(crate) names: (&'static str, &'static str),
// //     pub(crate) offspring_counts: VersionedCounts,
// //     pub(crate) survivor_counts: VersionedCounts,
// // }

// // impl<C> OffspringRecombineHandle<C>
// // where
// //     C: Chromosome + PartialEq + Clone,
// // {
// //     #[inline]
// //     pub fn create_with_species(
// //         &mut self,
// //         generation: usize,
// //         species: &[Species<C>],
// //         population: &mut Population<C>,
// //         metrics: &mut MetricSet,
// //     ) -> Population<C> {
// //         let mut lineage = self.lineage.write().unwrap();

// //         let mut species_scores = species
// //             .iter()
// //             .filter_map(|spec| spec.score())
// //             .collect::<Vec<_>>();

// //         if let Objective::Single(Optimize::Minimize) = &self.objective {
// //             species_scores.reverse();
// //         }

// //         let quotas = self.quotas_from_scores(&species_scores);

// //         let mut next_population = Population::with_capacity(self.count);
// //         for (species, count) in species.iter().zip(quotas.iter()) {
// //             let mut pop = population
// //                 .drain_species(species.id())
// //                 .collect::<Population<C>>();

// //             self.objective.sort(&mut pop);

// //             let time = std::time::Instant::now();

// //             let mut offspring = self
// //                 .selector
// //                 .select(&pop, &self.objective, *count)
// //                 .into_iter()
// //                 .map(|p| pop[p].clone())
// //                 .collect::<Population<C>>();

// //             metrics.upsert((self.names.0, offspring.len()));
// //             metrics.upsert((self.names.1, time.elapsed()));

// //             self.objective.sort(&mut offspring);

// //             self.alters.iter_mut().for_each(|alt| {
// //                 alt.alter(&mut offspring, &mut lineage, metrics, generation);
// //             });

// //             next_population.extend(offspring);
// //         }

// //         next_population
// //     }

// //     #[inline]
// //     fn quotas_from_scores(&self, scores: &[&Score]) -> Vec<usize> {
// //         let n = scores.len();
// //         if n == 0 || self.count == 0 {
// //             return vec![0; n];
// //         }

// //         let raw_scores = scores.iter().map(|s| s.as_f32()).collect::<Vec<f32>>();
// //         let mut min_score = raw_scores.iter().cloned().fold(f32::INFINITY, f32::min);
// //         if !min_score.is_finite() {
// //             min_score = 0.0;
// //         }

// //         let shifted = raw_scores
// //             .iter()
// //             .map(|s| (s - min_score).max(0.0))
// //             .collect::<Vec<f32>>();

// //         let sum = shifted.iter().sum::<f32>();

// //         if sum <= f32::EPSILON {
// //             let base = self.count / n;
// //             let mut quotas = vec![base; n];
// //             let mut remaining = self.count - base * n;
// //             let mut i = 0;
// //             while remaining > 0 {
// //                 quotas[i] += 1;
// //                 remaining -= 1;
// //                 i += 1;
// //             }

// //             return quotas;
// //         }

// //         let total = self.count as f32;

// //         let mut quotas = Vec::with_capacity(n);
// //         let mut fracs = Vec::with_capacity(n);
// //         let mut assigned = 0;

// //         for (idx, w) in shifted.iter().enumerate() {
// //             let p = *w / sum;
// //             let exact = p * total;
// //             let base = exact.floor() as usize;
// //             let frac = exact - base as f32;

// //             quotas.push(base);
// //             fracs.push((frac, idx));
// //             assigned += base;
// //         }

// //         let remaining = self.count.saturating_sub(assigned);
// //         fracs.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

// //         for (_, idx) in fracs.iter().take(remaining) {
// //             quotas[*idx] += 1;
// //         }

// //         quotas
// //     }
// // }

// // impl<C> RecombineStep<C>
// // where
// //     C: Chromosome + PartialEq + Clone,
// // {
// //     fn combined_create(
// //         &mut self,
// //         generation: usize,
// //         ecosystem: &mut Ecosystem<C>,
// //         metrics: &mut MetricSet,
// //     ) {
// //         let mut lineage = self.offspring_handle.lineage.write().unwrap();

// //         let pop_len = ecosystem.population().len();

// //         // Phase 1: run both selectors on the unmutated population.
// //         let s_timer = std::time::Instant::now();
// //         let s_indices = self.survivor_handle.selector.select(
// //             ecosystem.population(),
// //             &self.survivor_handle.objective,
// //             self.survivor_handle.count,
// //         );
// //         let s_elapsed = s_timer.elapsed();

// //         let o_timer = std::time::Instant::now();
// //         let o_indices = self.offspring_handle.selector.select(
// //             ecosystem.population(),
// //             &self.offspring_handle.objective,
// //             self.offspring_handle.count,
// //         );
// //         let o_elapsed = o_timer.elapsed();

// //         // Phase 2: aggregate per-idx counts into the two reusable buffers.
// //         self.offspring_handle.offspring_counts.begin(pop_len);
// //         for &idx in &o_indices {
// //             self.offspring_handle.offspring_counts.bump(idx);
// //         }

// //         self.offspring_handle.survivor_counts.begin(pop_len);
// //         for &idx in &s_indices {
// //             self.offspring_handle.survivor_counts.bump(idx);
// //         }

// //         // Phase 3: single descending walk. For each unique source idx with
// //         // total = s + o > 0, emit (total - 1) clones distributed to whichever
// //         // bucket still needs entries, then swap_remove the last one and place
// //         // it in whichever bucket has room.
// //         let mut survivors = Population::with_capacity(self.survivor_handle.count);
// //         let mut offspring = Population::with_capacity(self.offspring_handle.count);
// //         let pop = ecosystem.population_mut();

// //         for idx in (0..pop_len).rev() {
// //             let s = self.offspring_handle.survivor_counts.get(idx) as usize;
// //             let o = self.offspring_handle.offspring_counts.get(idx) as usize;
// //             let total = s + o;
// //             if total == 0 {
// //                 continue;
// //             }

// //             let (mut s_left, mut o_left) = (s, o);
// //             for _ in 0..total - 1 {
// //                 if s_left > 0 {
// //                     survivors.push(pop[idx].clone());
// //                     s_left -= 1;
// //                 } else {
// //                     offspring.push(pop[idx].clone());
// //                     o_left -= 1;
// //                 }
// //             }

// //             let moved = pop.swap_remove(idx);
// //             if s_left > 0 {
// //                 survivors.push(moved);
// //             } else {
// //                 let _ = o_left;
// //                 offspring.push(moved);
// //             }
// //         }

// //         // Metrics: keep both selectors' surface area unchanged.
// //         metrics.upsert((self.survivor_handle.names.0, survivors.len()));
// //         metrics.upsert((self.survivor_handle.names.1, s_elapsed));
// //         metrics.upsert((self.offspring_handle.names.0, offspring.len()));
// //         metrics.upsert((self.offspring_handle.names.1, o_elapsed));

// //         self.offspring_handle.objective.sort(&mut offspring);
// //         for alt in &mut self.offspring_handle.alters {
// //             alt.alter(&mut offspring, &mut lineage, metrics, generation);
// //         }

// //         let pop = ecosystem.population_mut();
// //         pop.clear();
// //         pop.extend(survivors);
// //         pop.extend(offspring);
// //     }
// // }
