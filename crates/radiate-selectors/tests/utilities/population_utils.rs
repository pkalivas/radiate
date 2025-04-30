use radiate_core::*;

#[allow(dead_code)]
pub fn float_population(num: usize) -> Population<FloatChromosome> {
    let mut population = Vec::with_capacity(num);

    for i in 0..num {
        let gene = (i as f32).into();
        let chromosome = FloatChromosome { genes: vec![gene] };
        let mut phenotype = Phenotype::from((vec![chromosome], 0));
        phenotype.set_score(Some(Score::from_f32(i as f32)));

        population.push(phenotype);
    }

    Population::new(population)
}

#[allow(dead_code)]
pub fn random_float_population(num: usize) -> Population<FloatChromosome> {
    let mut population = Vec::with_capacity(num);

    for _ in 0..num {
        let gene = FloatGene::from(0.0..100.0);
        let copy = gene.clone();
        let chromosome = FloatChromosome { genes: vec![gene] };
        let mut phenotype = Phenotype::from((vec![chromosome], 0));
        phenotype.set_score(Some(Score::from_f32(copy.allele)));

        population.push(phenotype);
    }

    Population::new(population)
}
