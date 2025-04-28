use radiate_core::{AlterResult, Chromosome, Crossover, random_provider};

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
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        Self { num_points, rate }
    }
}

impl<C: Chromosome> Crossover<C> for MultiPointCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, _: f32) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return 0.into();
        }

        let mut crossover_points = random_provider::indexes(0..length);
        crossover_points.sort();

        let selected_points = &crossover_points[..self.num_points];
        let mut offspring_one = Vec::with_capacity(length);
        let mut offspring_two = Vec::with_capacity(length);

        let mut current_parent = 1;
        let mut last_point = 0;

        for &point in selected_points {
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
            chrom_one.set(i, gene_one.clone());
            chrom_two.set(i, gene_two.clone());
        }

        self.num_points.into()
    }
}
