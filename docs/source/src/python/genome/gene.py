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
