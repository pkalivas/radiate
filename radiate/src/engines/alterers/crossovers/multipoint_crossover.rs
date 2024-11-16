use crate::engines::alterers::crossovers::crossover::Crossover;
use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::schema::subset;

const DEFAULT_NUM_POINTS: usize = 2;

pub struct MultiPointCrossover {
    pub num_points: usize,
    pub rate: f32,
}

impl MultiPointCrossover {
    pub fn new(rate: f32, num_points: usize) -> Self {
        Self { num_points, rate }
    }

    #[inline]
    pub fn swap<G: Gene<G, A>, A>(
        chrom_one: &mut Chromosome<G, A>,
        start: usize,
        end: usize,
        chrom_two: &mut Chromosome<G, A>,
        other_start: usize,
    ) {
        if other_start + (end - start) > chrom_one.len() {
            panic!(
                "Invalid index range: [{}, {})",
                other_start,
                other_start + (end - start)
            );
        }

        if start >= end {
            return;
        }

        for i in (end - start..0).rev() {
            let temp = chrom_one.get_gene(start + i);
            let other_gene = chrom_two.get_gene(other_start + i);

            let new_gene_one = temp.from_allele(&other_gene.allele());
            let new_gene_two = other_gene.from_allele(&temp.allele());

            chrom_one.set_gene(start + i, new_gene_one);
            chrom_two.set_gene(other_start + i, new_gene_two);
        }
    }
}

impl<G, A> Crossover<G, A> for MultiPointCrossover
where
    G: Gene<G, A>,
{
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut Chromosome<G, A>,
        chrom_two: &mut Chromosome<G, A>,
    ) -> i32 {
        let min_index = std::cmp::min(chrom_one.len(), chrom_two.len());
        let min_points = std::cmp::min(self.num_points, DEFAULT_NUM_POINTS);

        let mut cross_count = 0;
        let mut random = rand::thread_rng();
        let indexes = if min_points > 0 {
            subset::subset(min_index, min_points, &mut random)
        } else {
            Vec::new()
        };

        for i in 0..indexes.len() - 1 {
            let start = indexes[i] as usize;
            let end = indexes[i + 1] as usize;

            MultiPointCrossover::swap(chrom_one, start, end, chrom_two, start);
            cross_count += 1;
        }

        if indexes.len() % 2 == 1 {
            let index = indexes[indexes.len() - 1] as usize;

            cross_count += 1;
            MultiPointCrossover::swap(
                chrom_one,
                index,
                std::cmp::min(chrom_one.len(), chrom_two.len()),
                chrom_two,
                index,
            );
        }

        cross_count
    }
}
