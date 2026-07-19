import radiate as rd


# Setup (not shown): stand-in fitness functions / codec for the snippets below.
def my_fitness_fn(x):
    return 0.0


def fitness_function(individual):
    return 0.0


# --8<-- [start:basic_metrics]
import radiate as rd

# Create an engine with 3 chromosomes each with 2 genes (i.e. a 3x2 matrix)
engine = (
    rd.Engine.float([2, 2, 2], init_range=(0.0, 1.0))
    .fitness(my_fitness_fn)  # Single objective fitness function
    .limit(rd.Limit.generations(100))
    # ... other parameters ...
)

# Run the engine for 100 generations
result = engine.run()

# Get the metrics of the engine
metrics = result.metrics()  # MetricSet object
df = (
    metrics.to_polars()
)  # Convert metrics to a Polars DataFrame for analysis (if installed)
df = (
    metrics.to_pandas()
)  # Convert metrics to a Pandas DataFrame for analysis (if installed)

# Access specific metrics
carry_over = metrics[
    "pct.carryover"
].max()  # Maximum carryover rate throughout evolution

scores = metrics["scores"]
score_mean = scores.mean()
score_stddev = scores.stddev()
score_variance = scores.variance()
score_min = scores.min()
score_max = scores.max()
score_count = scores.count()
score_skew = scores.skew()
score_sum = scores.sum()

time = metrics["time"]

total_time = time.time_sum()
mean_time = time.time_mean()
stddev_time = time.time_stddev()
variance_time = time.time_variance()
min_time = time.time_min()
max_time = time.time_max()

# pretty-print the metrics dashboard
print(metrics.dashboard())
# --8<-- [end:basic_metrics]

# --8<-- [start:metric_tags]
import radiate as rd

# Create the evolution engine
engine = (
    rd.Engine.bit(10)
    .fitness(fitness_function)
    .limit(rd.Limit.score(0.01), rd.Limit.generations(1000))
)

# Run the engine
result = engine.run()

# Access the metrics from the result
metrics = result.metrics()

# Get tags for a specific metric
tags = metrics[
    "scores"
].tags()  # e.g., ['rd.Tag.SCORE', 'rd.Tag.STATISTIC', 'rd.Tag.DISTRIBUTION']

for metric in metrics.values_by_tag(rd.Tag.ALTERER):
    ...  # access all metrics related to alterers (crossover, mutation) ...
# --8<-- [end:metric_tags]
