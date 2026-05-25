import radiate as rd


# Setup (not shown): a stand-in error metric so the snippet is runnable.
def calculate_error(a: float, b: float) -> float:
    return abs(a) + abs(b)


# --8<-- [start:example]
import radiate as rd


# Define a fitness function that uses the decoded values
def fit(individual: list[float]) -> float:
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
offspring_selector = rd.Select.boltzmann(temp=4)

# Use tournament selection for survivors - individuals which will
# be passed down unchanged to the next generation
survivor_selector = rd.Select.tournament(k=3)

# Define the offspring fraction. This is the % of the population that
# will be created through mutation and crossover (offspring) vs passed down unchanged (survivors).
# The default is 80% offspring and 20% survivors, but here we'll use 50% for both.
fraction = 0.5

# Create the engine that will evolve a population of genomes with 2 genes in 1 chromosome, a
# fitness function, and selectors.
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), dtype=rd.Float32)
    .fitness(fit)
    .select(offspring=offspring_selector, survivor=survivor_selector, frac=fraction)
    .limit(rd.Limit.score(0.01), rd.Limit.generations(1000))
    # ... other parameters ...
)

# Run the engine
result = engine.run()
# --8<-- [end:example]
