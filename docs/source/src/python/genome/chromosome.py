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
