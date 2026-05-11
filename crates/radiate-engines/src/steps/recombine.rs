use crate::steps::EngineStep;
use radiate_core::Phenotype;
use radiate_core::{
    Alterer, Chromosome, Ecosystem, MetricSet, Objective, Optimize, Population, Score, Select,
};
use radiate_error::{Result, radiate_bail};
use radiate_utils::VersionedCounts;
use std::sync::Arc;

#[derive(Clone)]
pub struct SelectConfig<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) selector: Arc<dyn Select<C>>,
    pub(crate) names: (&'static str, &'static str),
}

impl<C: Chromosome> SelectConfig<C> {
    pub fn new(
        count: usize,
        selector: Arc<dyn Select<C>>,
        names: (&'static str, &'static str),
    ) -> Self {
        Self {
            count,
            selector,
            names,
        }
    }
}

#[derive(Clone)]
pub struct SurvivorConfig<C: Chromosome> {
    pub(crate) select: SelectConfig<C>,
}

impl<C: Chromosome> SurvivorConfig<C> {
    pub fn new(select: SelectConfig<C>) -> Self {
        Self { select }
    }
}

#[derive(Clone)]
pub struct OffspringConfig<C: Chromosome> {
    pub(crate) select: SelectConfig<C>,
    pub(crate) alters: Vec<Alterer<C>>,
}

impl<C: Chromosome> OffspringConfig<C> {
    pub fn new(select: SelectConfig<C>, alters: Vec<Alterer<C>>) -> Self {
        Self { select, alters }
    }
}

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor: SurvivorConfig<C>,
    pub(crate) offspring: OffspringConfig<C>,
    pub(crate) objective: Objective,
    pub(crate) survivor_counts: VersionedCounts,
    pub(crate) offspring_counts: VersionedCounts,
}

impl<C: Chromosome> RecombineStep<C> {
    pub fn new(
        survivor: SurvivorConfig<C>,
        offspring: OffspringConfig<C>,
        objective: Objective,
    ) -> Self {
        Self {
            survivor,
            offspring,
            objective,
            survivor_counts: VersionedCounts::new(),
            offspring_counts: VersionedCounts::new(),
        }
    }
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
        let new_members = if ecosystem.species().is_some() {
            self.create_with_species(generation, ecosystem, metrics)
        } else {
            self.combined_create(generation, ecosystem, metrics)
        };

        match new_members {
            Some((survivors, offspring)) => {
                let pop = ecosystem.population_mut();

                pop.clear();
                pop.extend(survivors);
                pop.extend(offspring);

                Ok(())
            }
            None => radiate_bail!("Failed to create new population during recombination step"),
        }
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
    /// selections (or in survivors only). On the other hand, some indices that
    /// fill only one bucket can be moved over from previous generation to
    /// current without cloning. In practice this can save ~20-50% of clones compared to
    /// a naive separate-walk approach.
    #[inline]
    fn combined_create(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Option<(Population<C>, Population<C>)> {
        let s_selector = &self.survivor.select;
        let o_selector = &self.offspring.select;

        let pop_slice = ecosystem.population().as_ref();
        let pop_len = pop_slice.len();

        let s_indices = self.timed_select(s_selector, pop_slice, metrics);
        let o_indices = self.timed_select(o_selector, pop_slice, metrics);

        self.offspring_counts.begin(pop_len);
        for &idx in o_indices.iter() {
            self.offspring_counts.bump(idx);
        }

        self.survivor_counts.begin(pop_len);
        for &idx in s_indices.iter() {
            self.survivor_counts.bump(idx);
        }

        let (survivors, mut offspring) = self.unioned_walk(ecosystem);

        self.objective.sort(&mut offspring);

        for alt in &mut self.offspring.alters {
            alt.alter(offspring.as_mut(), metrics, generation);
        }

        Some((survivors, offspring))
    }

    /// Species path: per-species reproduction. Survivors are selected globally
    /// via the survivor selector, then per-species offspring quotas drive
    /// scoped selection + alteration within each species' sub-pop. This follows
    /// a pretty similar approach to the above method, but we split the logic up by
    /// species, so each species essentially performs the above algorithm in it's own search space.
    #[inline]
    fn create_with_species(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Option<(Population<C>, Population<C>)> {
        let s_selector = &self.survivor.select;
        let o_selector = &self.offspring.select;

        let (species, population) = ecosystem.species_population_mut();
        let species = species?;

        population.sort_by(|a, b| a.species().cmp(&b.species()));

        let mut start = 0;
        let mut species_groups = Vec::with_capacity(species.len());
        let slice = population.as_ref();
        for chunk in slice.chunk_by(|a, b| a.species() == b.species()) {
            species_groups.push((chunk[0].species(), start..start + chunk.len()));
            start += chunk.len();
        }

        let mut species_scores = species
            .iter()
            .filter_map(|spec| spec.adj_score())
            .collect::<Vec<_>>();

        if let Objective::Single(Optimize::Minimize) = &self.objective {
            species_scores.reverse();
        }

        let quotas = self.quotas_from_scores(&species_scores);

        self.offspring_counts.begin(population.len());
        for (species, count) in species.iter().zip(quotas.iter()) {
            let range = species_groups
                .binary_search_by(|group| group.0.cmp(&species.id()))
                .ok()
                .map(|i| species_groups[i].1.clone())?;

            let mut sub_pop = &mut population[range.clone()];
            self.objective.sort(&mut sub_pop);

            let offspring = self.timed_select_count(o_selector, sub_pop, metrics, *count);

            for &idx in offspring.iter() {
                self.offspring_counts.bump(range.start + idx);
            }
        }

        let s_indices = self.timed_select(s_selector, population.as_ref(), metrics);

        self.survivor_counts.begin(population.len());
        for &idx in s_indices.iter() {
            self.survivor_counts.bump(idx);
        }

        let (survivors, mut offspring) = self.unioned_walk(ecosystem);

        let o_slice = offspring.as_mut();
        for sub_pop in o_slice.chunk_by_mut(|a, b| a.species() == b.species()) {
            let mut chunk = sub_pop;
            self.objective.sort(&mut chunk);

            for alt in &mut self.offspring.alters {
                alt.alter(chunk, metrics, generation);
            }
        }

        Some((survivors, offspring))
    }

    #[inline]
    fn timed_select(
        &self,
        select: &SelectConfig<C>,
        population: &[Phenotype<C>],
        metrics: &mut MetricSet,
    ) -> Vec<usize> {
        self.timed_select_count(select, population, metrics, select.count)
    }

    #[inline]
    fn timed_select_count(
        &self,
        select: &SelectConfig<C>,
        population: &[Phenotype<C>],
        metrics: &mut MetricSet,
        count: usize,
    ) -> Vec<usize> {
        let timer = std::time::Instant::now();
        let indices = select.selector.select(population, &self.objective, count);
        metrics.upsert(select.names.0, indices.len());
        metrics.upsert(select.names.1, timer.elapsed());
        indices
    }

    /// So, I was pulling my hair out over this for a bit because I knew it was possible but
    /// couldn't quite get it right. However, now that we've arrived at an elegant solution, this
    /// approach is pretty significant. The key insight is that we can interleave the survivor and offspring
    /// creation in a single walk over the union of selected indices, which allows us to save a clone for each
    /// index that appears in both selections. In practice this can save
    /// ~20-50% of clones compared to a naive approach.
    ///
    /// In other words:
    /// Single descending walk over the union of selected indices.
    /// For each unique source idx with total = s (survivors) + o (offspring) > 0, emit (total - 1)
    /// clones distributed to whichever bucket still needs entries, then
    /// swap_remove the last one and place it in whichever bucket has room.
    ///
    /// So total emissions = (total - 1) + 1 = total.
    /// Suppose s = 3, o = 2 for some idx (so total = 5):
    ///
    /// loop iteration 1: s_left=3 > 0, clone -> survivors[0], s_left=2
    /// loop iteration 2: s_left=2 > 0, clone -> survivors[1], s_left=1
    /// loop iteration 3: s_left=1 > 0, clone -> survivors[2], s_left=0
    /// loop iteration 4: s_left=0,     clone -> offspring[0]
    ///
    /// --- loop ends after total - 1 = 4 iterations ---
    ///
    /// swap_remove:      s_left=0,     move  -> offspring[1]
    ///
    /// Result: 3 clones to survivors, 1 clone & 1 move to offspring.
    /// This results in 4 clones total instead of 5. One deep Phenotype<C> clone saved
    /// per unique source idx.
    #[inline]
    fn unioned_walk(&self, ecosystem: &mut Ecosystem<C>) -> (Population<C>, Population<C>) {
        let mut survivors = Population::with_capacity(self.survivor.select.count);
        let mut offspring = Population::with_capacity(self.offspring.select.count);

        let pop = ecosystem.population_mut();
        let iter = self
            .survivor_counts
            .iter_pair_live_rev(&self.offspring_counts);

        for (idx, s, o) in iter {
            let mut s_left = s as usize;
            let total = s_left + o as usize;

            for _ in 0..total - 1 {
                if s_left > 0 {
                    survivors.push(pop[idx].clone());
                    s_left -= 1;
                } else {
                    offspring.push(pop[idx].clone());
                }
            }

            let moved = pop.swap_remove(idx);
            if s_left > 0 {
                survivors.push(moved);
            } else {
                offspring.push(moved);
            }
        }

        (survivors, offspring)
    }

    #[inline]
    fn quotas_from_scores(&self, scores: &[&Score]) -> Vec<usize> {
        let n = scores.len();
        if n == 0 || self.offspring.select.count == 0 {
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
            let base = self.offspring.select.count / n;
            let mut quotas = vec![base; n];
            let mut remaining = self.offspring.select.count - base * n;
            let mut i = 0;
            while remaining > 0 {
                quotas[i] += 1;
                remaining -= 1;
                i += 1;
            }

            return quotas;
        }

        let total = self.offspring.select.count as f32;

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

        let remaining = self.offspring.select.count.saturating_sub(assigned);
        fracs.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, idx) in fracs.iter().take(remaining) {
            quotas[*idx] += 1;
        }

        quotas
    }
}
