use radiate_core::*;

#[allow(dead_code)]
pub fn multi_obj_population(scores: Vec<Vec<f32>>) -> Population<FloatChromosome<f32>> {
    scores
        .into_iter()
        .map(|score| {
            let gene = FloatGene::from(0.0..1.0_f32);
            let chromosome = FloatChromosome::new(vec![gene]);
            let mut phenotype = Phenotype::from((vec![chromosome], 0));
            phenotype.set_score(Some(Score::from(score)));
            phenotype
        })
        .collect()
}

#[allow(dead_code)]
pub fn float_population(num: usize) -> Population<FloatChromosome<f32>> {
    let mut population = Vec::with_capacity(num);

    for i in 0..num {
        let gene = (i as f32).into();
        let chromosome = FloatChromosome::new(vec![gene]);
        let mut phenotype = Phenotype::from((vec![chromosome], 0));
        phenotype.set_score(Some(Score::from(i as f32)));

        population.push(phenotype);
    }

    Population::new(population)
}

#[allow(dead_code)]
pub fn random_float_population(num: usize) -> Population<FloatChromosome<f32>> {
    let mut population = Vec::with_capacity(num);

    for _ in 0..num {
        let gene = FloatGene::from(0.0..100.0);
        let copy = gene.clone();
        let chromosome = FloatChromosome::new(vec![gene]);
        let mut phenotype = Phenotype::from((vec![chromosome], 0));
        phenotype.set_score(Some(Score::from(*copy.allele())));

        population.push(phenotype);
    }

    Population::new(population)
}
