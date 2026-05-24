# Showcase (excluded from the run-test via the *_showcase.py suffix): launches the terminal
# UI (ui=True) and uses an intentionally-incomplete placeholder engine. Rendered into
# docs/source/misc/ui.md and include-validated by `mkdocs build --strict`.


def my_fitness_fn(val: list[list[bool]]) -> float:
    return sum(sum(row) for row in val)


# --8<-- [start:ui]
import radiate as rd

engine = (
    rd.Engine.bit(shape=[20, 20])
    .fitness(my_fitness_fn)
    .limit(rd.Limit.generations(10), rd.Limit.score(0.001))
    # ... configure your engine as normal ...
)

# Enable the UI by passing ui=True to run() Note that this will disable logging if log=True
result = engine.run(ui=True)
print(result)
# --8<-- [end:ui]
