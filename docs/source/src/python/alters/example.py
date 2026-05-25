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


# Define the alterers - these will be applied to the selected offspring
# to create new individuals. They will be applied in the order they are defined.
alters = [
    rd.Mutate.gaussian(rate=0.1),
    rd.Cross.blend(rate=0.8, alpha=0.5),
]

# Create the engine with the codec, fitness function, selectors, and alterers
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), dtype=rd.Float32)
    .fitness(fit)
    .select(
        offspring=rd.Select.boltzmann(temp=4),
        survivor=rd.Select.tournament(k=3),
        frac=0.5,
    )
    .alters(*alters)  # Add the alterers to the engine
    .limit(rd.Limit.score(0.01), rd.Limit.generations(1000))
    # ... other parameters ...
)

# Run the engine
result = engine.run()
# --8<-- [end:example]
