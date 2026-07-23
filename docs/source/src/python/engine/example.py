# Setup (not shown): trivial fitness for the snippets below. Small generation caps
# keep every example here fast and deterministic for tests.
def loop_fit(x):
    return 0.001


# --8<-- [start:single_limit]
import radiate as rd

engine = (
    rd.Engine.float(init_range=(0.0, 1.0)).fitness(loop_fit).limit(rd.Limit.generations(50))
)

result = engine.run()
# --8<-- [end:single_limit]

# --8<-- [start:combined_limits]
import radiate as rd

# Stops on whichever trips first - almost always the score target here, well before
# the 10,000-generation ceiling.
combined_engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(loop_fit)
    .limit(
        rd.Limit.generations(10_000),
        rd.Limit.score(0.01),
    )
)

combined_result = combined_engine.run()
# --8<-- [end:combined_limits]

# --8<-- [start:iterator_fallback]
import radiate as rd

# A Limit can only answer "should I stop?" - it can't hand you the intermediate state.
# Collecting the score history needs the real Generation at every step, which means
# iterating, not run().
iter_engine = (
    rd.Engine.float(init_range=(0.0, 1.0)).fitness(loop_fit).limit(rd.Limit.generations(50))
)

score_history: list[list[float]] = []
for epoch in iter_engine:
    score_history.append(epoch.score())

assert len(score_history) == 50
# --8<-- [end:iterator_fallback]
