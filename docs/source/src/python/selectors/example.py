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
    dtype=rd.Float32,  # Optional - default is Float64
)

# Use Boltzmann selection for offspring - individuals which
# will be used to create new individuals through mutation and crossover
offspring_selector = rd.BoltzmannSelector(temp=4)

# Use tournament selection for survivors - individuals which will
# be passed down unchanged to the next generation
survivor_selector = rd.TournamentSelector(k=3)

# Define the offspring fraction. This is the % of the population that
# will be created through mutation and crossover (offspring) vs passed down unchanged (survivors).
# The default is 80% offspring and 20% survivors, but here we'll use 50% for both.
fraction = 0.5

# Create the engine, fitness function, and selectors
# Note that the genome configuration below (rd.Engine.float(..)) is a
# shorthand for creating a codec and passing it to the engine constructor. The below is
# the same as the codec we created above, but built directly into the engine constructor for convenience.
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), dtype=rd.Float32)
    .fitness(fitness_function)
    .select(offspring_selector, survivor_selector, frac=fraction)
    # ... other parameters ...
)

# note the same engine can be built using a more traditional constructor pattern as such:
# engine = rd.Engine(
#     codec=codec,
#     fitness_func=fitness_function,
#     offspring_selector=offspring_selector,
#     survivor_selector=survivor_selector,
#     offspring_fraction=0.5,
#     # ... other parameters ...
# )

# Run the engine
result = engine.run(rd.ScoreLimit(0.01), rd.GenerationsLimit(1000))
# --8<-- [end:example]
