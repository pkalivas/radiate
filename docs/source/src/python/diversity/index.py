import radiate as rd


# Setup (not shown): stand-ins for the placeholders referenced in the snippets below.
your_codec = rd.FloatCodec(shape=2, init_range=(-1.0, 1.0))


def your_fitness_func(x):
    return sum(x)


diversity = rd.HammingDistance()


# --8<-- [start:diversity_basic]
import radiate as rd

# engine = rd.Engine(
#     codec=your_codec,
#     fitness_func=your_fitness_func,
#     diversity=diversity,
#     species_threshold=.5  # Default value
# )

# or using the fluent builder pattern:
engine = (
    rd.Engine(your_codec)
    .fitness(your_fitness_func)
    .diversity(diversity, species_threshold=0.5)  # Default value
    # ... other parameters ...
)
# --8<-- [end:diversity_basic]

# --8<-- [start:diversity_age]
import radiate as rd

# engine = rd.Engine(
#     codec=your_codec,
#     fitness_func=your_fitness_func,
#     diversity=diversity,
#     species_threshold=.5,  # Default value
#     max_species_age=20,  # Default value
# )

# or using the fluent builder pattern:
engine = (
    rd.Engine(your_codec)
    .fitness(your_fitness_func)
    .diversity(diversity, species_threshold=0.5)
    .age(max_species_age=20)  # Default value
    # ... other parameters ...
)
# --8<-- [end:diversity_age]
