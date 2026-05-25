# Showcase (excluded from the run-test via the *_showcase.py suffix): this writes checkpoint
# files to disk and re-runs an engine to perfect-match a target string — too slow / side-
# effecting for the fast suite. Rendered into docs/source/misc/checkpoint.md and its `--8<--`
# include is validated by `mkdocs build --strict`.

# --8<-- [start:checkpoint]
import radiate as rd

target = "Hello, Radiate!"


def fitness_func(x: list[str]) -> int:
    return sum(1 for i in range(len(target)) if x[i] == target[i])


engine = rd.Engine.char(len(target)).fitness(fitness_func)

result = engine.run(rd.Limit.score(len(target)), checkpoint=(10, "checks", "pkl"))

# load from checkpoint from generation 10
engine = (
    rd.Engine.char(len(target))
    .fitness(fitness_func)
    .load_checkpoint("checks/chckpnt_10.pkl")
)

result_from_checkpoint = engine.run(rd.Limit.score(len(target)))
# --8<-- [end:checkpoint]
