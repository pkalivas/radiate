# --8<-- [start:float_codec]
import radiate as rd

# scalar codec that decodes to a single float
codec = rd.FloatCodec(init_range=(-1.0, 1.0), bounds=(-10.0, 10.0))

# vector codec that decodes to a np.ndarray
codec = rd.FloatCodec(
    shape=5, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), use_numpy=True
)

# For a 3x2 matrix of parameters (like neural network weights)
codec = rd.FloatCodec(shape=(3, 2), init_range=(-0.1, 0.1), bounds=(-1.0, 1.0))
# -- or --
# supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes)
codec = rd.FloatCodec(
    shape=[2, 2, 2],
    init_range=(-0.1, 0.1),
    bounds=(-1.0, 1.0),
    use_numpy=True,
    dtype=rd.Float32,  # also adding dtype here - default is Float64
)
# --8<-- [end:float_codec]

# --8<-- [start:int_codec]
import radiate as rd

# For a single parameter
codec = rd.IntCodec(init_range=(0, 1), bounds=(-10, 10))

# For a list of parameters
codec = rd.IntCodec(shape=5, init_range=(-1, 1), bounds=(-10, 10))

# For a 3x2 matrix of parameters
codec = rd.IntCodec((3, 2), init_range=(-1, 1), bounds=(-10, 10))
# -- or --
# supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes)
codec = rd.IntCodec([2, 2, 2], init_range=(-1, 1), bounds=(-10, 10))

# The codec can also be created by directly:
# The chromosome will be a dense 4x5 matrix (4 rows with 5 colummns) of IntGenes that decodes to a 4x5 numpy array of np.int16 values.
codec = rd.IntCodec(
    shape=[5, 5, 5, 5],
    init_range=(0, 100),
    bounds=(-100, 200),
    use_numpy=True,
    dtype=rd.Int16,
)
# --8<-- [end:int_codec]

# --8<-- [start:char_codec]
import radiate as rd

# For a list of parameters
codec = rd.CharCodec(shape=5, char_set="abcdefghijklmnopqrstuvwxyz")

# For a matrix of chars
codec = rd.CharCodec((3, 2), char_set={"a", "b", "c", "d"})
# -- or --
# supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes) - use the default char_set
codec = rd.CharCodec([2, 2, 2])
# --8<-- [end:char_codec]

# --8<-- [start:bit_codec]
import radiate as rd

# For a list of parameters
codec = rd.BitCodec(5)

# For a matrix of bools
codec = rd.BitCodec((3, 2))
# -- or --
# supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes)
codec = rd.BitCodec([2, 2, 2])
# --8<-- [end:bit_codec]

# --8<-- [start:permutation_codec]
import radiate as rd

# For a list of unique items
codec = rd.PermutationCodec(alleles=[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
# This will produce a Genotype<PermutationChromosome> with 1 PermutationChromosome which
# holds 10 unique genes (0-9) in a random order.
genotype = codec.encode()
# Decode to a list of unique items
decoded = codec.decode(genotype)
# decoded will be a list of unique items from the original alleles.
# e.g. [3, 0, 7, 1, 9, 2, 5, 6, 4, 8]
# Note: The order of the decoded items will be the same as the order of the
# genes in the PermutationChromosome, which is a random permutation of the original alleles.
# --8<-- [end:permutation_codec]
