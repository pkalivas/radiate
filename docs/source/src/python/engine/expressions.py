import radiate as rd


# Setup (not shown): a trivial fitness so the engine examples below terminate fast and
# deterministically. A small *non-zero* constant keeps the best score below the 0.01
# limits (so they trip quickly) while keeping derived stats like stddev/mean well-defined.
def my_fitness_fn(x):
    return 0.001


# --8<-- [start:building]
import radiate as rd

# Select a metric by name. By default this reads the last recorded value.
score = rd.Expr.select("scores.best")

# A literal constant
threshold = rd.Expr.lit(0.01)

# The current generation index
gen = rd.Expr.generation()
# --8<-- [end:building]

# --8<-- [start:aggregations]
import radiate as rd

score = rd.Expr.select("scores.best")

score.last()  # last recorded value (default)
score.mean()  # running mean of all values seen
score.stddev()  # standard deviation
score.min()  # running minimum
score.max()  # running maximum
score.sum()  # running sum
score.var()  # variance
score.skew()  # skewness
score.count()  # number of values seen
score.slope()  # linear slope over accumulated values
score.unique()  # deduplicated collection

# Rolling window: aggregate over the last N values only
score.rolling(50).mean()  # mean of the last 50 values
score.rolling(50).stddev()  # std dev of the last 50
score.rolling(100).slope()  # slope over a 100-generation window
# --8<-- [end:aggregations]

# --8<-- [start:comparisons]
import radiate as rd

score = rd.Expr.select("scores.best")

# Comparisons — Python operators work directly. Each produces a Bool expression.
below = score < 0.01
at_most = score <= 0.01
above = score > 0.99
at_least = score >= 0.99
equal = score == 0.5
not_equal = score != 0.5

# Boolean logic combines Bool expressions
both = (score < 0.01) & (rd.Expr.select("index") > 50)  # and
either = (score < 0.01) | (rd.Expr.select("time") > 10.0)  # or
negated = ~(score < 0.01)  # not

# A range check is just two comparisons (an inclusive "between")
in_range = (score >= 0.0) & (score <= 1.0)
# --8<-- [end:comparisons]

# --8<-- [start:arithmetic]
import radiate as rd

a = rd.Expr.select("scores.best")
b = rd.Expr.select("score.volatility")

added = a + b
subtracted = a - b
multiplied = a * 2.0
divided = a / b
squared = a**2
negated = -a
abs(a)
a.clamp(0.0, 1.0)
# --8<-- [end:arithmetic]

# --8<-- [start:conditional]
import radiate as rd

# If the best score is below 0.01, use its mean; otherwise use a fallback literal
expr = (
    rd.Expr.when(rd.Expr.select("scores.best") < 0.01)
    .then(rd.Expr.select("scores.best").mean())
    .otherwise(rd.Expr.lit(1.0))
)
# --8<-- [end:conditional]

# --8<-- [start:schedule]
import radiate as rd

# Compute a rolling stddev but only report it every 10 generations;
# otherwise return the last value.
expr = (
    rd.Expr.every(10)
    .then(rd.Expr.select("scores.best").rolling(10).stddev())
    .otherwise(rd.Expr.select("scores.best"))
)
# --8<-- [end:schedule]

# --8<-- [start:querying]
import radiate as rd

# Mean of the time metric, interpreted as a duration
rd.Expr.select("time").time().mean()

# Count of evaluations as a number
rd.Expr.select("count.evaluation").count()
# --8<-- [end:querying]

# --8<-- [start:limit_expr]
import radiate as rd

engine = rd.Engine.float(10, init_range=(-5.0, 5.0)).fitness(my_fitness_fn).minimizing()

# Stop when the best score has been below 0.01 on average over the last 50 generations
stop_expr = rd.Expr.select("scores.best").rolling(50).mean() < 0.01

result = engine.limit(rd.Limit.expr(stop_expr)).run()
# --8<-- [end:limit_expr]

# Rebuild the engine + stop_expr (not shown) so the combined-limit snippet below is runnable.
engine = rd.Engine.float(10, init_range=(-5.0, 5.0)).fitness(my_fitness_fn).minimizing()
stop_expr = rd.Expr.select("scores.best").rolling(50).mean() < 0.01

# --8<-- [start:limit_combined]
result = engine.limit(
    rd.Limit.expr(stop_expr),
    rd.Limit.generations(5000),  # hard ceiling
).run()
# --8<-- [end:limit_combined]

# --8<-- [start:derived_metrics]
import radiate as rd

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
    .limit(rd.Limit.generations(500))
)

# These metrics are now available in every generation result
result = engine.run()
metrics = result.metrics()
print(metrics["score_trend"].value_last())
print(metrics["score_cv"].value_last())
# --8<-- [end:derived_metrics]

# --8<-- [start:derived_metrics_limit]
# Register a trend metric, then stop when it flattens
engine = (
    rd.Engine.float(10, init_range=(-5.0, 5.0))
    .fitness(my_fitness_fn)
    .minimizing()
    .metrics(score_trend=rd.Expr.select("scores.best").rolling(50).slope())
    .limit(
        rd.Limit.expr(abs(rd.Expr.select("score_trend")) < 0.0001),
        rd.Limit.generations(5000),
    )
)

result = engine.run()
# --8<-- [end:derived_metrics_limit]

# --8<-- [start:dynamic_rates]
import radiate as rd

# Start aggressive, decay as volatility drops
dynamic_rate = rd.Expr.select("score.volatility").rolling(20).mean().clamp(0.01, 0.5)

engine = (
    rd.Engine.float(10, init_range=(-5.0, 5.0))
    .fitness(my_fitness_fn)
    .minimizing()
    .alters(
        rd.Mutate.gaussian(rate=dynamic_rate),
        rd.Cross.blend(rate=0.5),
    )
)
# --8<-- [end:dynamic_rates]
