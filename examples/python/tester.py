import radiate as rd

# metrics = rd.MetricSet(one=list(range(10)), two=list(range(10, 20)), three=4)
# metrics.upsert("four", 2)
# metrics.upsert("four", 3)
# metrics.upsert("four", 4)
# metrics.upsert("four", 5)


# print(metrics.dashboard())
def my_fitness_fn(x):
    return 0.001


score_trend = rd.Expr.select("scores.best").rolling(20).slope().debug()
score_cv = (
    rd.Expr.select("scores.best").rolling(20).stddev()
    / rd.Expr.select("scores.best").rolling(20).mean()
)

engine = (
    rd.Engine.float(10, init_range=(-5.0, 5.0))
    .fitness(my_fitness_fn)
    .minimizing()
    .metrics(
        score_trend=score_trend,
        score_cv=score_cv,
    )
    .limit(rd.Limit.generations(1000))
)


result = engine.run()
metrics = result.metrics()
print(metrics["score_trend"].value_last())
print(metrics["score_cv"].value_last())
