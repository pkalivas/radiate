use super::{Alter, AlterAction, Crossover, EngineCompoment};

use crate::{random_provider, Chromosome};

/// The `MultiPointCrossover` is a crossover method that takes two chromosomes and crosses them
/// by selecting multiple points in the chromosome and swapping the genes between the two chromosomes.
/// The number of points to swap is determined by the `num_points` parameter and must be between 1 and the
/// length of the chromosome. Note, in most cases having more than 2 points is not useful and actually
/// reduces the effectiveness of the crossover. However, it can be useful in some cases so it is allowed.
///
/// This is the traditional crossver method used by genetic algorithms. It is a
/// simple method that can be used with any type of gene.
pub struct MultiPointCrossover {
    num_points: usize,
    rate: f32,
}

impl MultiPointCrossover {
    /// Create a new instance of the `MultiPointCrossover` with the given rate and number of points.
    /// The rate must be between 0.0 and 1.0, and the number of points must be between 1 and the length
    /// of the chromosome.
    pub fn new(rate: f32, num_points: usize) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("Rate must be between 0 and 1");
        }

        Self { num_points, rate }
    }
}

impl EngineCompoment for MultiPointCrossover {
    fn name(&self) -> &'static str {
        "MultiPointCrossover"
    }
}

impl<C: Chromosome> Alter<C> for MultiPointCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<C: Chromosome> Crossover<C> for MultiPointCrossover {
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return 0;
        }

        let mut crossover_points: Vec<usize> = (1..length).collect();
        random_provider::shuffle(&mut crossover_points);

        let selected_points = &crossover_points[..self.num_points];

        let mut sorted_points = selected_points.to_vec();
        sorted_points.sort();

        let mut offspring_one = Vec::with_capacity(length);
        let mut offspring_two = Vec::with_capacity(length);

        let mut current_parent = 1;
        let mut last_point = 0;

        for &point in &sorted_points {
            if current_parent == 1 {
                offspring_one.extend_from_slice(&chrom_one.as_ref()[last_point..point]);
                offspring_two.extend_from_slice(&chrom_two.as_ref()[last_point..point]);
            } else {
                offspring_one.extend_from_slice(&chrom_two.as_ref()[last_point..point]);
                offspring_two.extend_from_slice(&chrom_one.as_ref()[last_point..point]);
            }

            current_parent = 3 - current_parent;
            last_point = point;
        }

        if current_parent == 1 {
            offspring_one.extend_from_slice(&chrom_one.as_ref()[last_point..]);
            offspring_two.extend_from_slice(&chrom_two.as_ref()[last_point..]);
        } else {
            offspring_one.extend_from_slice(&chrom_two.as_ref()[last_point..]);
            offspring_two.extend_from_slice(&chrom_one.as_ref()[last_point..]);
        }

        for i in 0..length {
            let gene_one = &offspring_one[i];
            let gene_two = &offspring_two[i];
            chrom_one.set_gene(i, gene_one.clone());
            chrom_two.set_gene(i, gene_two.clone());
        }

        self.num_points as i32
    }
}
