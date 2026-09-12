import radiate as rd


# Setup (not shown in the docs): a stand-in error metric so the snippet is runnable.
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

# Create the evolution engine
engine = (
    rd.Engine(codec)
    .fitness(fit)
    .limit(rd.Limit.score(0.01))
    .limit(rd.Limit.generations(1000))
)

# Here is where the engine's helper function is used. Again,
# same parameters as above, just wrapped in a more fluid interface.
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), dtype=rd.Float32)
    .fitness(fit)
    .limit(rd.Limit.score(0.01))
    .limit(rd.Limit.generations(1000))
)

# Run the engine
result = engine.run()
# --8<-- [end:example]
