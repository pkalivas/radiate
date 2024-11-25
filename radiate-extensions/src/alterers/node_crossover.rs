use radiate::engines::alterers::Crossover;
use radiate::engines::genome::*;
use radiate::{Alterer, RandomProvider};

use crate::node::Node;
use crate::operations::op::Ops;

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
    pub fn alterer(rate: f32) -> Alterer<Node<T>, Ops<T>> {
        Alterer::Crossover(Box::new(Self {
            rate,
            _marker: std::marker::PhantomData,
        }))
    }
}

impl<T> Crossover<Node<T>, Ops<T>> for NodeCrossover<T>
where
    T: Clone + PartialEq + Default,
{
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "Node Crossover"
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut Chromosome<Node<T>, Ops<T>>,
        chrom_two: &mut Chromosome<Node<T>, Ops<T>>,
    ) -> i32 {
        let rate = self.cross_rate();
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if RandomProvider::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                if gene_one.value.arity() != gene_two.value.arity()
                    || gene_one.node_type() != gene_two.node_type()
                {
                    continue;
                }

                let new_gene_one = gene_one.from_allele(gene_two.allele());
                let new_gene_two = gene_two.from_allele(gene_one.allele());

                chrom_one.set_gene(i, new_gene_one);
                chrom_two.set_gene(i, new_gene_two);

                cross_count += 1;
            }
        }

        cross_count
    }
}
