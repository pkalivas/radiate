use radiate::*;
use std::f32;

mod plotting;
mod robot;

const NUM_GENES: usize = 6;
const GENE_RANGE: std::ops::Range<f32> = -5.0..5.0;

fn main() {
    random_provider::set_seed(5522);

    let novelty = NoveltySearch::new(robot::behavior_descriptor)
        .k(10)
        .threshold(0.6)
        .archive_size(1000)
        .cosine_distance();

    let engine = GeneticEngine::builder()
        .codec(FloatChromosome::from((NUM_GENES, GENE_RANGE)))
        .fitness_fn(novelty)
        .survivor_selector(TournamentSelector::new(3))
        .offspring_selector(BoltzmannSelector::new(4.0))
        .alter(alters!(
            BlendCrossover::new(0.1, 0.5),
            GaussianMutator::new(0.1)
        ))
        .build();

    engine
        .iter()
        .logging()
        .take(2000)
        .last()
        .inspect(|result| visualize(result.clone()));
}

fn visualize(result: Generation<FloatChromosome, Vec<f32>>) {
    let population = result.population().clone();

    let mut robots = Vec::new();
    // Only visualize first 6 individuals - At this point the population is sorted by fitness (best (1st) to worst (last))
    for indiv in population.iter().take(6) {
        let genes: Vec<f32> = indiv
            .genotype()
            .iter()
            .flat_map(|chrom| chrom.iter().map(|g| *g.allele()))
            .collect();

        robots.push(genes);
    }

    plotting::visualize_behaviors_grid(&robots);
}
