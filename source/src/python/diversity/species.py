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

# --8<-- [start:target_species_count]
import radiate as rd

# `target_species` is an alternative to `species_threshold` that tries to maintain a certain number of
# species. The engine will adjust the threshold up or down as needed to try to meet the target count.
engine = (
    rd.Engine.float(2)
    .fitness(your_fitness_func)
    .diversity(rd.Dist.euclidean(), target_species=4)
)

# This is equivalent to setting the `species_threshold`
# (previous section) to an expression like so (this is actually what the engine does
# under the hood when you set `target_species`):
initial_threshold = 0.5  # <- note that this is the default species_threshold
target_species = 4

species_threshold = (
    rd.Expr.when(rd.Expr.select("index") < 2)
    .then(initial_threshold)
    .otherwise(
        (rd.Expr.select("species.count").error(target_species) * 0.05)
        + rd.Expr.select("species.threshold")
    )
)
# --8<-- [end:target_species_count]
