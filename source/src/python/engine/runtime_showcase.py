import radiate as rd


# This file is excluded from the run-test suite (`*_showcase.py`): the snippet calls
# `engine.run(..., ui=True, checkpoint=...)`, which launches the terminal UI and writes
# checkpoint files — not suitable for the fast unit suite. The include is still build-checked.
def my_fitness_fn(x):
    return 0.0


# --8<-- [start:iterator_run_options]
import radiate as rd

engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(my_fitness_fn)
    .limit(rd.Limit.generations(100))
)

# `run()` also accepts logging, a terminal UI, and checkpointing
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
# --8<-- [end:iterator_run_options]
