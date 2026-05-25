import radiate as rd


# This file is excluded from the run-test suite (`*_showcase.py`): the snippet calls
# `engine.run(..., ui=True, checkpoint=...)`, which launches the terminal UI and writes
# checkpoint files — not suitable for the fast unit suite. The include is still build-checked.
def my_fitness_fn(x):
    return 0.0


# --8<-- [start:running]
import radiate as rd

# Create an engine
engine = (
    rd.Engine.float(init_range=(0.0, 1.0)).fitness(my_fitness_fn)
    # ... other parameters ...
)

# use a simple for loop to iterate through 100 generations
for epoch in engine:
    if epoch.index() >= 100:
        break
    print(f"Generation {epoch.index()}: Score = {epoch.score()}")

# just use the next() function to get the next epoch
while True:
    # the 'next' function calls the iterator internally & is very efficient, the only clones that happen
    # will be on the first call to a method that requires ownership of the epoch data.
    epoch = next(engine)
    if epoch.index() >= 100:
        break
    print(f"Generation {epoch.index()}: Score = {epoch.score()}")

# --- or using the engine's Run method with limits ---

# Limits - run until a score target is reached
score_limit = rd.Limit.score(0.01)
generations_limit = rd.Limit.generations(100)
seconds_limit = rd.Limit.seconds(60)
# window and threshold for convergence - how close the scores must be over the window to consider convergence
convergence_limit = rd.Limit.convergence(window=50, threshold=0.01)
# metric based limit - by metric name:
# stop after the evaluation count metric reaches 1000. Note that the metric function is
# a predicate that takes the metric value and returns a boolean indicating whether
# to stop or not, this allows for more complex stopping conditions based on metrics.
metric_limit = rd.Limit.metric("count.evaluation", lambda metric: metric.sum() >= 1000)

# Add the limits directly to the engine
# Create an engine
engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(my_fitness_fn)
    .limit(
        score_limit,
        generations_limit,
        seconds_limit,
        convergence_limit,
        metric_limit,
    )
    # ... other parameters ...
)

# Or pass the limits to the run method.
# Log the progress of the engine to the console
result = engine.run(
    log=True,
    ui=True,  # Enable terminal UI - if enabled, log is ignored
    checkpoint=(
        10,
        "checkpoint",
        "pkl",
    ),
    # checkpoint every 10 generations to the folder "checkpoint" in pickle
    # format - can be loaded with the .load_checkpoint() method on the engine
)
# --8<-- [end:running]
