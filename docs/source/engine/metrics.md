# Metrics

Metric collection in radiate is interwoven into every aspect of the evolutionary process. It uses the [Kahan summation algorithm](https://en.wikipedia.org/wiki/Kahan_summation_algorithm) paired with [Welford's one-pass online algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm) for fast, accurate, and numerically stable computation of statistics. All of this combined provides robust and reliable metric tracking throughout the evolutionary process. Using the `MetricSet` (a collection of independent `Metric`s) we can collect a whole host of statistics that span the entire evolutionary process allowing us to gain deep insights into the evolutionary dynamics.

---

## MetricSet

The `MetricSet` is an object (struct) provided to the user in two main forms:

1. On the engine's `Generation` - given to the user after each epoch or each pass of the evolution process.
2. Through the engine's eventing system. Various events emit metric data allowing the user to track metrics or derive their own in real-time.

A `metric` is essentially a statistic with a name and some extra metadata attached to it. Each one exposes the usual summary statistics — last value, count, min, max, mean, sum, standard deviation, variance, and skewness. Kurtosis is also available on the Rust side; the Python binding doesn't expose it.

There are a few different types of metrics that can be collected:

1. **Numeric Metric**: A plain old metric that collects single point numeric data each generation and aggregates it over the _entire_ evolutionary run. For example, the `pct.diversity` metric collects the diversity ratio each generation and folds it into the running statistic alongside every prior generation's value. 
2. **Duration Metric**: A metric that collects timing information for a step of the pipeline, named `step.<phase>.time` (e.g. `step.evaluate.time`, `step.recombine.time`). Each generation's timing is folded into the running statistic alongside every prior generation's. Duration metrics expose their values as a `Duration` through a dedicated set of accessors (`time_sum`/`time_mean`/`time_stddev`/… in Python, `Metric::times()` in Rust) rather than the plain numeric ones described below.
3. **Distribution Metric**: A metric that collects a distribution of data each generation, replacing the previous generation's data. For example, the `scores` metric collects the scores of all individuals in the population each generation, replacing the previous generation's scores. This means that each generation, the metric reflects only the _current_ generation's state, nothing before it.

## Collection

Along with the default metrics, each component will also collect metrics for the operations it performs. For example, each `Alterer` and `Selector` will collect metrics and be identified by their name. A few types of metrics will only be included if the parts of the engine which produce them are included. For instance, **`species`** level metrics will only be collected if the engine is configured to use species-based diversity. Also, **`front`** level metrics will only be collected if the engine is configured for multi-objective optimization. 

**Note** this does not include all possible metrics. Certain metrics are only collected under specific conditions or configurations, such as when using species-based diversity or multi-objective optimization.

## Default Metrics

Metrics collected by default (always included):

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `index`             | The index of the generation. This is pretty much just for reference if you want to keep a log per generation. |
| `time`              | The time taken for the evolution process.                                   |
| `scores`            | The scores (fitness) of all the individuals evolved throughout the evolution process. |
| `scores.best`       | The best score found so far over the course of the run. |
| `scores.evenness`   | Pielou evenness of the population's fitness distribution, in `[0, 1]`. `~1.0` means fitness is spread evenly across distinct scores (healthy, exploring); `~0.0` means the population has collapsed onto a plateau (premature convergence). |
| `scores.gini`       | Gini coefficient of the population's fitness distribution, in `[0, 1]` — a measure of fitness *inequality* / effective selection pressure. `0.0` is perfect equality (fully converged); high (`~0.7+`) means a small elite holds most of the fitness mass. |
| `age`               | The age of all the individuals in the `Ecosystem` throughout the evolution process. |
| `genome.size`       | The size of each genome over the evolution process. This is usually static and doesn't change. |
| `replace.age`       | The number of individuals replaced based on age. |
| `replace.invalid`   | The number of individuals replaced based on invalid structure (e.g. Bounds). |
| `unique.members`   | The number of individuals in the `Ecosystem` with a distinct identity this generation. This is not about score or genotype equality — it drops below population size only when the same individual occupies more than one population slot (e.g. selection with replacement cloning a survivor into multiple spots). |
| `unique.scores`    | The number of unique scores in the `Ecosystem`. |
| `new.children`     | The number of new children created each generation through either mutation or crossover (or both). |
| `count.survivor`   | The number of individuals that survived to the next generation - summation throughout the evolution process. |
| `count.evaluation` | The total number of evaluations performed per generation. |
| `count.stagnation` | The number of consecutive generations since the best score last improved. Resets to `0` on any improvement. |
| `pct.carryover`   | The rate at which unique individuals are carried over to the next generation - `survivor_count` per generation / population size. |
| `pct.diversity`  | The ratio of `unique.members` to the size of the `Ecosystem` — not a score-based measure; see `unique.members` above. |
| `score.volatility` | The volatility of the scores in the `Ecosystem`. This is calculated as the standard deviation of the scores / mean. **Only emitted for single-objective runs** — under multi-objective optimization the underlying bare `scores` metric isn't populated (it's written per-dimension instead), so this metric is omitted entirely rather than suffixed. |
| `score.improvement` | The improvement of the best score from the previous generation to the current generation - either a 1 or 0 each generation. |

A few default metrics are only collected when the relevant data exists or a specific feature is opted into:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `genome.size.score.corr` | Pearson correlation between genome size and fitness across the population, in `[-1, 1]` — the bloat signal. Only emitted when genome length actually varies (variable-length GP genomes); for fixed-length genomes there is no size variance and the metric is omitted. |
| `filter.unique.scores` | The number of individuals replaced this generation by a `UniqueScoreFilter` — an opt-in stagnation-recovery filter that, once the best score has gone stagnant past a configured threshold, replaces individuals whose score duplicates another member's. `0` whenever the filter isn't registered or hasn't triggered. |

!!! note "Multi-objective naming"

    The per-dimension metrics — `scores`, `scores.best`, `scores.evenness`, `scores.gini`, `unique.scores`, and `genome.size.score.corr` — are emitted under their bare name for single-objective runs. Under **multi-objective** optimization they gain a numeric suffix per objective instead (`scores.0`, `scores.1`, …, `scores.best.0`, `scores.best.1`, and so on).

## Multi-objective Metrics

Additional metrics collected when using multi-objective optimization:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `front.additions`    | The number of members added to the Pareto front each generation. |
| `front.removals`     | The number of members removed from the Pareto front each generation. |
| `front.size`         | The size of the Pareto front each generation. |
| `front.comparisons`  | The number of comparisons made to update the Pareto front each generation. |
| `front.filters`      | The number of times the Pareto front was filtered each generation. |
| `front.entropy`      | The entropy of the Pareto front throughout the evolution process - only calculated every 10 generations (its kinda an expensive calculation). |

## Species-based Metrics

Additional metrics collected when using species-based diversity:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `species.count`    | The number of `species` in the `Ecosystem`. |
| `species.new`  | The number of `species` created in the `Ecosystem`. |
| `species.new.ratio` | The ratio of new species created each generation. |
| `species.fail.empty`     | The number of `species` that have died (emptied out) in the `Ecosystem`. |
| `species.age`      | The age of all the `species` in the `Ecosystem`. |
| `species.fail.age` | The count of species that have failed based on age each generation. |
| `species.size`     | The distribution of species sizes (number of members per species) each generation. |
| `species.distance` | The distribution of compatibility distances used when assigning members to species. |
| `species.threshold` | The current compatibility threshold used to decide species membership. |
| `species.evenness` | The evenness of the species distribution in the `Ecosystem`. |
| `species.largest_share` | The share of the largest species in the `Ecosystem`. |


## Accessing Metrics

These can be accessed through the `metrics()` method of the `Generation` object, which returns a `MetricSet`. Each individual `Metric` can be accessed by its name, and the various statistical measures can be accessed through the methods of the `Metric` object. Additionally, the `MetricSet` provides a `dashboard()` method that pretty-prints all the metrics in a user-friendly format.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/metrics.py:basic_metrics"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/metrics.rs:basic_metrics"
    ```

---

## Tags 

All metrics have a sort of metadata which identifies them based on their characteristics or where they originate from. This can be used to filter and group metrics based on similar traits. For example, metrics related to time will have the `time` tag, while metrics related to `mutators` will have the `mutator` tag.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/engine/metrics.py:metric_tags"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/engine/metrics.rs:metric_tags"
    ```

Tags available:

| Tag                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `Selector`          | Metrics related to selection mechanisms.                                    |
| `Alterer`           | Metrics related to alteration mechanisms (mutation, crossover).            |
| `Mutator`           | Metrics specifically related to mutation operations.                        |
| `Crossover`         | Metrics specifically related to crossover operations.                       |
| `Species`           | Metrics related to species-based diversity.                                 |
| `Failure`           | Metrics related to failures (e.g., invalid individuals).                    |
| `Age`               | Metrics related to age-based operations.                                    |
| `Front`             | Metrics related to multi-objective optimization fronts.                     |
| `Derived`           | Metrics that are derived from other metrics.                                |
| `Other`             | Miscellaneous metrics that don't fit into other categories.                 |
| `Statistic`        | Metrics that provide statistical measures.                                  |
| `Time`              | Metrics related to time measurements.                                       |
| `Distribution`     | Metrics that describe distributions (e.g., scores, ages).                        |
| `Score`             | Metrics specifically related to fitness scores.                             |
| `Rate`              | Metrics that represent rates (e.g., carryover rate).                                |
| `Step`              | Metrics related to the different steps or phases of the evolutionary process (evaluation, recombination, filtering, etc.).    |
| `Expr`             | Custom metrics or expressions supplied by the user.   |