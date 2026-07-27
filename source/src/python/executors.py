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


# Here we can see how to use the parallel executors in python.
# Radiate will throw an error if you try to use a parallel executor
# in a non-free-threaded interpreter or if the GIL is enabled.
# a good way to check is by using radiate's utlity function, rd._GIL_ENABLED,
# which will be True if the GIL is enabled and False if it is not.
print(f"GIL enabled: {rd._GIL_ENABLED}")

# Building the engine dynamically can do something like this:
# .parallel(num_workers: int | None = None) method.
# If num_workers is None, it will use rayon's global thread pool, otherwise
# it will use a fixed sized worker pool with the specified number of workers.
engine = (
    rd.Engine.float(2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), dtype=rd.Float32)
    .fitness(fit)
    .select(
        offspring=rd.Select.boltzmann(temp=4),
        survivor=rd.Select.tournament(k=3),
        frac=0.5,
    )
    .alters(
        rd.Mutate.gaussian(rate=0.1),
        rd.Cross.blend(rate=0.8, alpha=0.5),
    )
    .limit(rd.Limit.score(0.01), rd.Limit.generations(1000))
    # ... other parameters ...
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
result = engine.run()
# --8<-- [end:example]
