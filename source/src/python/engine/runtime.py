# Setup (not shown): trivial fitness for the snippets below. A small generation cap
# keeps every example here fast and deterministic for tests.
def loop_fit(x):
    return 0.001


# --8<-- [start:iterator_basic]
import radiate as rd

engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(loop_fit)
    .limit(rd.Limit.generations(5))
)

# The engine is itself an iterator over `Generation` epochs
for epoch in engine:
    print(f"Generation {epoch.index()}: Score = {epoch.score()}")
# --8<-- [end:iterator_basic]

# --8<-- [start:iterator_next]
import radiate as rd

engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(loop_fit)
    .limit(rd.Limit.generations(5))
)

# Or drive it manually with the builtin `next()` - useful for interleaving other work
# between generations. Every call builds a fresh `Generation` snapshot; prefer `run()`
# (below) when you don't need to inspect each generation as it happens.
while True:
    epoch = next(engine)
    if epoch.index() >= 5:
        break
    print(f"Generation {epoch.index()}: Score = {epoch.score()}")
# --8<-- [end:iterator_next]

# --8<-- [start:iterator_convenience]
import radiate as rd

engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(loop_fit)
    .limit(rd.Limit.generations(5))
)

# `run()` is the convenience wrapper: it drives the same Limit-driven loop internally
# and just hands back the final epoch.
convenience_result: rd.Generation[float, float] = engine.run()
# --8<-- [end:iterator_convenience]
