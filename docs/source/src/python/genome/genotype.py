# --8<-- [start:genotype]
import radiate as rd

# Create a genotype with a single FloatChromosome and a 5 FloatGenes
genotype = rd.Genotype(rd.Chromosome.float(length=5, init_range=(-1.0, 1.0)))

# Create a genotype with a single FloatChromosome and a single FloatGene with a
# randomly generated allele between -1.0 and 1.0
genotype = rd.Genotype(rd.Chromosome([rd.Gene.float(init_range=(-1.0, 1.0))]))

# Create a genotype with multiple chromosomes of lengths 5, 15, and 3
three_chromosome_genotype = rd.Genotype(
    [
        rd.Chromosome.float(length=5, init_range=(-1.0, 1.0)),
        rd.Chromosome.float(length=15, init_range=(-1.0, 1.0)),
        rd.Chromosome.float(length=3, init_range=(-1.0, 1.0)),
    ]
)
# --8<-- [end:genotype]
