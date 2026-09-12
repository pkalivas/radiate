# Mirror of docs/source/genome/index.md — every Python snippet on that page lives here.

# --8<-- [start:float_gene]
import radiate as rd

# Create a float gene that can evolve between -1.0 and 1.0 but
# must stay within -10.0 to 10.0 during evolution
gene = rd.Gene.float(
    allele=0.5,  # Current value
    init_range=(
        -1.0,
        1.0,
    ),  # Initial range - if no allele, the allele will be randomly initialized within this range
    bounds=(-10.0, 10.0),  # Evolution bounds
    dtype=rd.Float32,  # Optional, defaults to rd.Float64
)
# --8<-- [end:float_gene]

# --8<-- [start:int_gene]
import radiate as rd

# Create an integer gene that can evolve between -100 and 100
gene = rd.Gene.int(
    allele=42,  # Current value
    init_range=(
        -10,
        10,
    ),  # Initial range - if no allele, the allele will be randomly initialized within this range
    bounds=(-100, 100),  # Evolution bounds - optional, defaults to init_range
    dtype=rd.Int16,  # Optional, defaults to rd.Int64
)
# --8<-- [end:int_gene]

# --8<-- [start:bit_gene]
import radiate as rd

# Create an bit gene with an allele of True - if the allele isn't specified, it will
# be randomly initialized to True or False
gene = rd.Gene.bit(allele=True)
# --8<-- [end:bit_gene]

# --8<-- [start:char_gene]
import radiate as rd

# Create a character gene with an allele of 'A'
gene = rd.Gene.char(allele="A")

# Create a character gene with a randomly generated allele from the set 'abc'
gene = rd.Gene.char(char_set=set("abc"))

# Create a character gene from a set of chars
gene = rd.Gene.char(char_set=set("abc"))
gene = rd.Gene.char(char_set={"a", "b", "c"})
# --8<-- [end:char_gene]

# --8<-- [start:float_chromosome]
import radiate as rd

# Create a float chromosome with 5 genes, each initialized to a random value between -1.0 and 1.0
chromosome = rd.Chromosome.float(
    length=5,  # Number of genes in the chromosome
    init_range=(-1.0, 1.0),  # Initial range for gene alleles
    bounds=(
        -10.0,
        10.0,
    ),  # Optional, bounds for gene alleles during evolution - defaults to init_range
    dtype=rd.Float32,  # Optional, defaults to rd.Float64
)

# Create a float chromosome with specific genes
genes = [
    rd.Gene.float(allele=0.1),
    rd.Gene.float(allele=0.2),
    rd.Gene.float(allele=0.3),
]
chromosome_with_genes = rd.Chromosome.float(genes=genes)
# --8<-- [end:float_chromosome]

# --8<-- [start:int_chromosome]
import radiate as rd

# Create an integer chromosome with 5 genes, each initialized to a random value between -10 and 10
chromosome = rd.Chromosome.int(
    length=5,  # Number of genes in the chromosome
    init_range=(-10, 10),  # Initial range for gene alleles
    bounds=(
        -100,
        100,
    ),  # Optional, bounds for gene alleles during evolution - defaults to init_range
    dtype=rd.Int16,  # Optional, defaults to rd.Int64
)

# Create an integer chromosome with specific genes
genes = [rd.Gene.int(allele=1), rd.Gene.int(allele=2), rd.Gene.int(allele=3)]
chromosome_with_genes = rd.Chromosome(genes)
# --8<-- [end:int_chromosome]

# --8<-- [start:bit_chromosome]
import radiate as rd

# Create a bit chromosome with 5 genes, each initialized to a random value of True or False
chromosome = rd.Chromosome.bit(length=5)
# --8<-- [end:bit_chromosome]

# --8<-- [start:char_chromosome]
import radiate as rd

# Create a character chromosome with 5 genes, each initialized to a random character from the ASCII printable characters
chromosome = rd.Chromosome.char(length=5)

chromosome_with_set = rd.Chromosome.char(length=5, char_set=set("abc"))
# --8<-- [end:char_chromosome]

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
