# Setup (not shown): trivial fitness for the snippets below.
def loop_fit(x):
    return 0.001


# --8<-- [start:limits_each]
import radiate as rd

score_limit = rd.Limit.score(0.01)
generations_limit = rd.Limit.generations(100)
seconds_limit = rd.Limit.seconds(60)
# window and threshold for convergence - how close the scores must be over the
# window to consider convergence
convergence_limit = rd.Limit.convergence(window=50, threshold=0.01)
# --8<-- [end:limits_each]

# --8<-- [start:limits_combined]
import radiate as rd

combined_engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(loop_fit)
    # The engine stops as soon as ANY one of these is reached
    .limit(
        rd.Limit.generations(5),
        rd.Limit.seconds(30),
        rd.Limit.score(0.01),
    )
)

combined_result = combined_engine.run()
# --8<-- [end:limits_combined]
