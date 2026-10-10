use radiate::prelude::*;

const NUM_BITS: usize = 10000;

fn main() -> RadiateResult<()> {
    let engine = GeneticEngine::builder()
        .codec(PackedBitCodec::new(NUM_BITS))
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .raw_fitness_fn(|geno: &Genotype<PackedBitChromosome>| geno.count_ones())
        .alter(alters!(
            PackedBitMultiPointCrossover::new(0.7, 2),
            BitFlipMutator::new(0.2 / NUM_BITS as f64)
        ))
        .build();

    let result = engine.iter().logging().until_score(NUM_BITS).last()?;

    println!("{}", result.metrics().dashboard());

    Ok(())
}
