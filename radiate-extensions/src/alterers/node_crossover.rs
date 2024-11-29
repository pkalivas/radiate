use crate::NodeChromosome;
use radiate::alter::AlterType;
use radiate::engines::genome::*;
use radiate::{random_provider, Alter};

pub struct NodeCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub rate: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NodeCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub fn new(rate: f32) -> Self {
        Self {
            rate,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Alter<NodeChromosome<T>> for NodeCrossover<T>
where
    T: Clone + PartialEq + Default,
{
    fn name(&self) -> &'static str {
        "Node Crossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut NodeChromosome<T>,
        chrom_two: &mut NodeChromosome<T>,
    ) -> i32 {
        let rate = self.rate();
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                if gene_one.value.arity() != gene_two.value.arity()
                    || gene_one.node_type() != gene_two.node_type()
                {
                    continue;
                }

                let new_gene_one = gene_one.with_allele(gene_two.allele());
                let new_gene_two = gene_two.with_allele(gene_one.allele());

                chrom_one.set_gene(i, new_gene_one);
                chrom_two.set_gene(i, new_gene_two);

                cross_count += 1;
            }
        }

        cross_count
    }
}
