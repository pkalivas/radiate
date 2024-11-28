use radiate::*;

#[allow(dead_code)]
pub fn float_population(num: usize) -> Population<FloatChromosome> {
    let mut population = Vec::with_capacity(num);

    for i in 0..num {
        let gene: FloatGene = (i as f32).into();
        let chromosome = FloatChromosome::from_genes(vec![gene]);
        let mut phenotype = Phenotype::from_chromosomes(vec![chromosome], 0);
        phenotype.set_score(Some(Score::from_f32(i as f32)));

        population.push(phenotype);
    }

    Population::from_vec(population)
}

#[allow(dead_code)]
pub fn random_float_population(num: usize) -> Population<FloatChromosome> {
    let mut population = Vec::with_capacity(num);

    for _ in 0..num {
        let gene: FloatGene = FloatGene::new(0.0, 100.0);   
        let copy = gene.clone();
        let chromosome = FloatChromosome::from_genes(vec![gene]);
        let mut phenotype = Phenotype::from_chromosomes(vec![chromosome], 0);
        phenotype.set_score(Some(Score::from_f32(copy.allele)));

        population.push(phenotype);
    }

    Population::from_vec(population)
}