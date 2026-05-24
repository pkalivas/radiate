import radiate as rd


# Setup (not shown in the docs): a stand-in error metric so the snippet is runnable.
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

# Create the evolution engine
engine = rd.Engine(
    codec=codec,
    fitness_func=fitness_function,
    # ... other parameters ...
)

# note the same engine can be built using a fluent builder pattern as such:
engine = rd.Engine.float(
    2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), dtype=rd.Float32
).fitness(fitness_function)

# Theres no real difference between the two, but
# radiate as a whole is moving towards the builder pattern.
# It allows for much better type hinting and is more intuitive to use.
# Both methods will be supported for the foreseeable future however, so feel free to use either one.

# Run the engine
result = engine.run(rd.Limit.score(0.01), rd.Limit.generations(1000))
# --8<-- [end:example]
