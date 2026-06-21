import radiate as rd

# Setup (not shown): stand-ins for the placeholders referenced in the snippets below.
your_codec = rd.FloatCodec(shape=2, init_range=(-1.0, 1.0))


def your_fitness_func(x):
    return sum(abs(v) for v in x)


# --8<-- [start:diversity_basic]
import radiate as rd

engine = (
    rd.Engine(your_codec)
    .fitness(your_fitness_func)
    # A distance measure turns speciation on; the threshold sets how close
    # two individuals must be (per the measure) to share a species.
    .diversity(rd.Dist.euclidean(), species_threshold=0.5)
    # ... other parameters ...
)
# --8<-- [end:diversity_basic]
