import radiate as rd


# Setup (not shown): a stand-in error metric so the snippet is runnable.
def calculate_error(a: float, b: float) -> float:
    return abs(a) + abs(b)


# --8<-- [start:example]
import radiate as rd


# Define a fitness function that uses the decoded values
def fitness_function(individual: list[float]) -> float:
    # Calculate how well these parameters fit your data
    a = individual[0]
    b = individual[1]
    return calculate_error(a, b)  # Your error calculation here


# Create a codec for two parameters (a and b)
codec = rd.FloatCodec(
    shape=2,  # We need two parameters: a and b
    init_range=(-1.0, 1.0),  # Start with values between -1 and 1
    bounds=(-10.0, 10.0),  # Allow evolution to modify the values between -10 and 10
)

# Use Boltzmann selection for offspring - individuals which
# will be used to create new individuals through mutation and crossover
offspring_selector = rd.BoltzmannSelector(temp=4)

# Use tournament selection for survivors - individuals which will
# be passed down unchanged to the next generation
survivor_selector = rd.TournamentSelector(k=3)

# Define the alterers - these will be applied to the selected offspring
# to create new individuals. They will be applied in the order they are defined.
alters = [
    rd.GaussianMutator(rate=0.1),
    rd.BlendCrossover(rate=0.8, alpha=0.5),
]

# Define the diversity measure
diversity = rd.HammingDistance()  # or rd.EuclideanDistance() for continuous problems

# Build the engine with the fluent builder pattern, attaching the diversity measure,
# species threshold, and max species age:
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0))
    .fitness(fitness_function)
    .select(offspring_selector, survivor_selector)
    .alters(*alters)
    .diversity(
        diversity, species_threshold=0.5
    )  # Add the diversity measure and species threshold
    .age(max_species_age=20)  # Add the max species age
    # ... other parameters ...
)

# The same configuration can also be passed straight to the engine constructor:
# engine = rd.Engine(
#     codec=codec,
#     fitness_func=fitness_function,
#     offspring_selector=offspring_selector,
#     survivor_selector=survivor_selector,
#     alters=alters,
#     diversity=diversity,
#     species_threshold=0.5,
#     max_species_age=20,
#     # ... other parameters ...
# )

# Run the engine
result = engine.run(rd.ScoreLimit(0.01), rd.GenerationsLimit(1000))
# --8<-- [end:example]
