use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome};

pub struct MultiPointCrossover {
    pub num_points: usize,
    pub rate: f32,
}

impl MultiPointCrossover {
    pub fn new(rate: f32, num_points: usize) -> Self {
        Self { num_points, rate }
    }
}

impl<C: Chromosome> Alter<C> for MultiPointCrossover {
    fn name(&self) -> &'static str {
        let result = format!("MultiPointCrossover ({})", self.num_points);
        Box::leak(result.into_boxed_str())
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return 0;
        }

        let mut crossover_points: Vec<usize> = (1..length).collect();
        random_provider::shuffle(&mut crossover_points);
        let num_points = 2;
        let selected_points = &crossover_points[..num_points];

        let mut sorted_points = selected_points.to_vec();
        sorted_points.sort();

        let mut offspring_one = Vec::with_capacity(length);
        let mut offspring_two = Vec::with_capacity(length);

        let mut current_parent = 1;
        let mut last_point = 0;

        for &point in &sorted_points {
            if current_parent == 1 {
                offspring_one.extend_from_slice(&chrom_one.get_genes()[last_point..point]);
                offspring_two.extend_from_slice(&chrom_two.get_genes()[last_point..point]);
            } else {
                offspring_one.extend_from_slice(&chrom_two.get_genes()[last_point..point]);
                offspring_two.extend_from_slice(&chrom_one.get_genes()[last_point..point]);
            }

            current_parent = 3 - current_parent;
            last_point = point;
        }

        if current_parent == 1 {
            offspring_one.extend_from_slice(&chrom_one.get_genes()[last_point..]);
            offspring_two.extend_from_slice(&chrom_two.get_genes()[last_point..]);
        } else {
            offspring_one.extend_from_slice(&chrom_two.get_genes()[last_point..]);
            offspring_two.extend_from_slice(&chrom_one.get_genes()[last_point..]);
        }

        for i in 0..length {
            let gene_one = &offspring_one[i];
            let gene_two = &offspring_two[i];
            chrom_one.set_gene(i, gene_one.clone());
            chrom_two.set_gene(i, gene_two.clone());
        }

        num_points as i32
    }
}
