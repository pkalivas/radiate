import radiate as rd


# Setup (not shown): stand-ins for the placeholders referenced in the snippets below.
your_codec = rd.FloatCodec(shape=2, init_range=(-1.0, 1.0))


def your_fitness_func(x):
    return sum(abs(v) for v in x)


def your_char_fit_func(x):
    return sum(ord(c) for c in x)


# --8<-- [start:threshold]
import radiate as rd

engine = (
    rd.Engine.char(10)
    .fitness(your_char_fit_func)
    # A distance measure turns speciation on; the threshold sets how close
    # two individuals must be (per the measure) to share a species.
    .diversity(rd.Dist.hamming(), species_threshold=0.5)
)
# --8<-- [end:threshold]

# --8<-- [start:dynamic_threshold]
import radiate as rd

# `species_threshold` accepts a `Rate`, so it can change over generations.
# Here it widens from 0.3 to 0.9 across the first 100 generations: start
# fine-grained (many small species), then coarsen to encourage convergence.
engine = (
    rd.Engine.float(2)
    .fitness(your_fitness_func)
    .diversity(
        rd.Dist.euclidean(),
        species_threshold=rd.Rate.linear(start=0.3, end=0.9, duration=100),
    )
)
# --8<-- [end:dynamic_threshold]

# --8<-- [start:age]
import radiate as rd

engine = (
    rd.Engine.float(2)
    .fitness(your_fitness_func)
    .diversity(rd.Dist.euclidean(), species_threshold=0.5)
    # A species that survives this many generations without improving its best
    # score is culled, and its members sit out crossover/mutation that generation.
    .age(max_species_age=25)
)
# --8<-- [end:age]
