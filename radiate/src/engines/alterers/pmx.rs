use super::{AlterAction, CrossoverAction, EngineAlterer, EngineCompoment};
use crate::subset::subset;
use crate::{Chromosome, PermutationChromosome};

pub struct PMXCrossover {
    rate: f32,
}

impl PMXCrossover {
    pub fn new(rate: f32) -> Self {
        PMXCrossover { rate }
    }
}

impl EngineCompoment for PMXCrossover {
    fn name(&self) -> &'static str {
        "PMX Crossover"
    }
}

impl<A: PartialEq + Clone> EngineAlterer<PermutationChromosome<A>> for PMXCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<PermutationChromosome<A>> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<A: PartialEq + Clone> CrossoverAction<PermutationChromosome<A>> for PMXCrossover {
    fn cross_chromosomes(
        &self,
        chrom_one: &mut PermutationChromosome<A>,
        chrom_two: &mut PermutationChromosome<A>,
    ) -> i32 {
        let length = std::cmp::min(chrom_one.genes.len(), chrom_two.genes.len());
        if length < 2 {
            return 0;
        }

        let subset = subset(chrom_one.genes.len(), 2);
        let start = subset[0] as usize;
        let end = subset[1] as usize;

        let mut offspring_one = chrom_one.genes.clone();
        let mut offspring_two = chrom_two.genes.clone();

        // PMX Crossover
        offspring_one[start..(end + 1)].clone_from_slice(&chrom_two.genes[start..(end + 1)]);
        offspring_two[start..(end + 1)].clone_from_slice(&chrom_one.genes[start..(end + 1)]);

        for i in 0..length {
            if i < start || i > end {
                let mut gene_one = chrom_one.get_gene(i);
                let mut gene_two = chrom_two.get_gene(i);

                while offspring_one[start..=end].contains(gene_one) {
                    let index = chrom_two.genes.iter().position(|g| g == gene_one).unwrap();
                    gene_one = chrom_one.get_gene(index);
                }

                while offspring_two[start..=end].contains(gene_two) {
                    let index = chrom_one.genes.iter().position(|g| g == gene_two).unwrap();
                    gene_two = chrom_two.get_gene(index);
                }

                offspring_one[i] = gene_one.clone();
                offspring_two[i] = gene_two.clone();
            }
        }

        chrom_one.genes = offspring_one;
        chrom_two.genes = offspring_two;

        2
    }
}

// impl<A: PartialEq + Clone> Alter<PermutationChromosome<A>> for PMXCrossover {
//     fn name(&self) -> &'static str {
//         "PMX Crossover"
//     }

//     fn rate(&self) -> f32 {
//         self.rate
//     }

//     fn alter_type(&self) -> AlterType {
//         AlterType::Crossover
//     }

//     fn cross_chromosomes(
//         &self,
//         chrom_one: &mut PermutationChromosome<A>,
//         chrom_two: &mut PermutationChromosome<A>,
//     ) -> i32 {
//         let length = std::cmp::min(chrom_one.genes.len(), chrom_two.genes.len());
//         if length < 2 {
//             return 0;
//         }

//         let subset = subset(chrom_one.genes.len(), 2);
//         let start = subset[0] as usize;
//         let end = subset[1] as usize;

//         let mut offspring_one = chrom_one.genes.clone();
//         let mut offspring_two = chrom_two.genes.clone();

//         // PMX Crossover
//         offspring_one[start..(end + 1)].clone_from_slice(&chrom_two.genes[start..(end + 1)]);
//         offspring_two[start..(end + 1)].clone_from_slice(&chrom_one.genes[start..(end + 1)]);

//         for i in 0..length {
//             if i < start || i > end {
//                 let mut gene_one = chrom_one.get_gene(i);
//                 let mut gene_two = chrom_two.get_gene(i);

//                 while offspring_one[start..=end].contains(gene_one) {
//                     let index = chrom_two.genes.iter().position(|g| g == gene_one).unwrap();
//                     gene_one = chrom_one.get_gene(index);
//                 }

//                 while offspring_two[start..=end].contains(gene_two) {
//                     let index = chrom_one.genes.iter().position(|g| g == gene_two).unwrap();
//                     gene_two = chrom_two.get_gene(index);
//                 }

//                 offspring_one[i] = gene_one.clone();
//                 offspring_two[i] = gene_two.clone();
//             }
//         }

//         chrom_one.genes = offspring_one;
//         chrom_two.genes = offspring_two;

//         2
//     }
// }
