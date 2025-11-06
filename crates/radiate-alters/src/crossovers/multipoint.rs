use radiate_core::{AlterResult, Chromosome, Crossover, random_provider};

/// The [MultiPointCrossover] is a crossover method that takes two chromosomes and crosses them
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
    /// Create a new instance of the [MultiPointCrossover] with the given rate and number of points.
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
        let one = chrom_one.genes_mut();
        let two = chrom_two.genes_mut();
        crossover_multi_point(one, two, self.num_points).into()
    }
}

#[inline]
pub fn crossover_multi_point<G>(
    chrom_one: &mut [G],
    chrom_two: &mut [G],
    num_points: usize,
) -> usize {
    let length = std::cmp::min(chrom_one.len(), chrom_two.len());

    if length < 2 {
        return 0;
    }

    let mut crossover_points = random_provider::shuffled_indices(0..length);

    let selected_points = &mut crossover_points[..num_points];
    selected_points.sort();

    let mut current_parent = 1;
    let mut last_point = 0;

    for i in selected_points {
        if current_parent == 1 {
            chrom_one[last_point..*i].swap_with_slice(&mut chrom_two[last_point..*i]);
        }

        current_parent = 3 - current_parent;
        last_point = *i;
    }

    if current_parent == 1 {
        chrom_one[last_point..].swap_with_slice(&mut chrom_two[last_point..]);
    }

    num_points
}

#[inline]
pub fn crossover_single_point<G>(chrom_one: &mut [G], chrom_two: &mut [G]) -> usize {
    let length = std::cmp::min(chrom_one.len(), chrom_two.len());

    if length < 2 {
        return 0;
    }

    let crossover_point = random_provider::range(1..length);
    chrom_one[crossover_point..].swap_with_slice(&mut chrom_two[crossover_point..]);

    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossover_multi_point() {
        let mut chrom_one = vec![0; 10];
        let mut chrom_two = vec![1; 10];

        let points = crossover_multi_point(&mut chrom_one, &mut chrom_two, 2);

        assert_eq!(chrom_one.len(), 10);
        assert_eq!(chrom_two.len(), 10);
        assert_eq!(points, 2);
    }
}
