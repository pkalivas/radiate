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

# Here we can see how to use the parallel executors in python.
# Radiate will throw an error if you try to use a parallel executor in a non-free-threaded interpreter or if the GIL is enabled.
# a good way to check is by using radiate's utlity function, rd._GIL_ENABLED, which will be True if the GIL is enabled and False if it is not.

print(f"GIL enabled: {rd._GIL_ENABLED}")

# Building the engine dynamically can do something like this:
# .parallel(num_workers: int | None = None) method.
# If num_workers is None, it will use rayon's global thread pool, otherwise
# it will use a fixed sized worker pool with the specified number of workers.
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0))
    .fitness(fitness_function)
    .select(rd.Select.boltzmann(temp=4), rd.Select.tournament(k=3))
    .alters(
        rd.Mutate.gaussian(rate=0.1),
        rd.Cross.blend(rate=0.8, alpha=0.5),
    )
    .diversity(rd.Dist.hamming(), 0.5)
    .age(max_species_age=20)
    # ... other builder methods ...
)

if rd._GIL_ENABLED:
    print("GIL is enabled - have to use Serial Executor.")
else:
    print("GIL is not enabled, using parallel executor.")
    engine = engine.parallel(
        num_workers=4
    )  # Use a fixed sized worker pool with 4 workers
    # --- or ----
    # engine = engine.parallel()  # Use a worker pool with rayon's global thread pool

# Run the engine
result = engine.run(rd.Limit.score(0.01), rd.Limit.generations(1000))
# --8<-- [end:example]
