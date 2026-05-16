# Metrics

Metric collection in radiate is interwoven into every aspect of the evolutionary process. It uses the [Kahan summation algorithm](https://en.wikipedia.org/wiki/Kahan_summation_algorithm) paired with [Welford's one-pass online algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm) for fast, accurate, and numerically stable computation of statistics. All of this combined provides robust and reliable metric tracking throughout the evolutionary process. Using the `MetricSet` (a collection of independent `Metric`s) we can collect a whole host statistics that span the entire evolutionary process allowing us to gain deep insights into the evolutionary dynamics.

---

## MetricSet

The `MetricSet` is an object (struct) provided to the user in two main forms:

1. On the engine's `Generation` - given to the user after each epoch or each pass of the evolution process.
2. Through the engine's eventing system. Various events emit metric data allowing the user to track metrics or derive their own in real-time.

A `metric` is essentially a statistic with a name and some extra metadata attached it. The `Statistic` exposes a number of different statistical measures that can be used to summarize the data, such as, `last_value`, `count`, `min`, `max`, `mean`, `sum`, `variance`, `std_dev`, `skewness`, and `kurtosis`.

There are a few differeny types of metrics that can be collected:

1. **Numeric Metric**: A plain old metric that collects single point numeric data each generation and aggregates it over the _entire_ evolutionary run. For example, the `rate.diversity` metric collects the diversity rate each generation and adds it to the previous generation's metrics. 
2. **Duration Metric**: A metric that collects timing information for various components of the engine. For example, the `time.evaluation` metric collects the time taken to perform evaluations each generation and adds it to the previous generation's metrics. When accessed, it should be noted that when calling `metric.time()` the underyling statistic will convert the numerical data to a `Duration` object, which provides methods for accessing the time in different units (e.g., seconds, milliseconds, etc.).
3. **Distribution Metric**: A metric that collects a distribution of data each generation, replacing the previous generation's data. For example, the `scores` metric collects the scores of all individuals in the population each generation, replacing the previous generation's scores. This means that each generation, the metric reflects only the _current_ generation's state, nothing before it.

## Collection

Along with the default metrics, each component will also collect metrics for the operations it performs. For example, each `Alterer` and `Selector` will collect metrics and be identified by their name. A few types of metrics will only be included if the parts of the engine which produce them are included. For instance, **`species`** level metrics will only be collected if the engine is configured to use species-based diversity. Also, **`front`** level metrics will only be collected if the engine is configured for multi-objective optimization. 

**Note** this does not include all possible metrics. Certain metrics are only collected under specific conditions or configurations, such as when using species-based diversity or multi-objective optimization.

## Default Metrics

Metrics collected by default (always included):

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `time`              | The time taken for the evolution process.                                   |
| `scores`            | The scores (fitness) of all the individuals evolved throughout the evolution process. |
| `age`               | The age of all the individuals in the `Ecosystem` throughout the evolution process. |
| `age.replace`      | The number of individuals replaced based on age. |
| `invalid.replace`  | The number of individuals replaced based on invalid structure (e.g. Bounds) |
| `size.genome`      | The size of each genome over the evolution process. This is usually static and doesn't change. |
| `unique.members`   | The number of unique members in the `Ecosystem`. |
| `unique.scores`    | The number of unique scores in the `Ecosystem`. |
| `new.children`     | The number of new children created each generation through either mutation or crossover (or both). |
| `count.survivors`   | The number of individuals that survived to the next generation - summation throughout the evolution process. |
| `rate.carryover`   | The rate at which unique individuals are carried over to the next generation - `survivor_count` per generation / population size. |
| `count.evaluation` | The total number of evaluations performed per generation. |
| `rate.diversity`  | The ratio of unique scores to the size of the `Ecosystem`. |
| `score.volatility` | The volatility of the scores in the `Ecosystem`. This is calculated as the standard deviation of the scores / mean. |
| `score.improvement` | The improvement of the best score from the previous generation to the current generation - either a 1 or 0 each generation. |
| `index`             | The index of the generation. This is pretty much just for reference if you want to keep a log per generation. |

## Multi-objective Metrics

Additional metrics collected when using multi-objective optimization:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `new.front`            | The number of members added to the Pareto front each generation. | 
| `front.entropy`  | The entropy of the Pareto front throughout the evolution process - only calculated every 10 generations (its kinda an expensive calculation). |
| `invalid.front`  | The number of members removed from the Pareto front each generation. |
| `front.comparisons`  | The number of comparisons made to update the Pareto front each generation. |
| `size.front`  | The size of the Pareto front each generation. |   
| `front.filters`  | The number of times the Pareto front was filtered each generation. |

## Species-based Metrics

Additional metrics collected when using species-based diversity:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `count.species`    | The number of `species` in the 'Ecosystem`. |
| `new.species`  | The number of `species` created in the `Ecosystem`. |
| `invalid.species`     | The number of `species` that have died in the `Ecosystem`. |
| `age.species`      | The age of all the `species` in the `Ecosystem`. |
| `age.species.fail` | The count of species that have failed based on age each generation. |
| `species.evenness` | The evenness of the species distribution in the `Ecosystem`. |
| `species.largest_share` | The share of the largest species in the `Ecosystem`. |
| `new.species.ratio` | The ratio of new species created each generation. |


## Accessing Metrics

These can be accessed through the `metrics()` method of the `Generation` object, which returns a `MetricSet`. Each individual `Metric` can be accessed by its name, and the various statistical measures can be accessed through the methods of the `Metric` object. Additionally, the `MetricSet` provides a `dashboard()` method that pretty-prints all the metrics in a user-friendly format.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create an engine with 3 chromosomes each with 2 genes (i.e. a 3x2 matrix)
    engine = (
        rd.Engine.float([2, 2, 2], init_range=(0.0, 1.0))
        .fitness(my_fitness_fn)  # Single objective fitness function
        # ... other parameters ...
    )

    # Run the engine for 100 generations
    result = engine.run(rd.GenerationsLimit(100))

    # Get the metrics of the engine
    metrics = result.metrics()  # MetricSet object
    df = metrics.to_polars()  # Convert metrics to a Polars DataFrame for analysis (if installed)
    df = metrics.to_pandas()  # Convert metrics to a Pandas DataFrame for analysis (if installed)

    # Access specific metrics
    carry_over = metrics['carryover_rate'].max() # Maximum carryover rate throughout evolution
    
    scores = metrics["scores"] 
    score_mean = scores.mean()  
    score_stddev = scores.stddev()  
    score_variance = scores.variance()  
    score_min = scores.min()  
    score_max = scores.max()  
    score_count = scores.count()  
    score_skew = scores.skew()  
    score_sum = scores.sum()

    time = metrics["time"].time_sum() 

    total_time = time.time_sum()  
    mean_time = time.time_mean()  
    stddev_time = time.time_stddev()  
    variance_time = time.time_variance()  
    min_time = time.time_min()  
    max_time = time.time_max()  

    # pretty-print the metrics dashboard
    print(metrics.dashboard())
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    // --- set up the engine ---

    let result = engine.run(|ctx| {
        // get the scroe metric from the generation context
        let temp = ctx.metrics().get("scores").unwrap();
        // get the standard deviation of the score distribution
        let std = temp.value_std_dev();
        
        std < 0.01 // Example condition to stop the engine
    });

    // Access the metrics from the result
    let metrics: MetricSet = result.metrics();

    // pretty-print the metrics dashboard
    println!("{}", metrics.dashboard())
    ```

---

## Tags 

All metrics have a sort of metadata which identifies them based on their characteristics or where they originate from. This can be used to filter and group metrics based on similar traits. For example, metrics related to time will have the `time` tag, while metrics related to `mutators` will have the `mutator` tag.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create the evolution engine
    engine = rd.Engine(
        codec=codec,
        fitness_func=fitness_function,
        # ... other parameters ...
    )

    # Run the engine
    result = engine.run(rd.ScoreLimit(0.01), rd.GenerationsLimit(1000))

    # Access the metrics from the result
    metrics = result.metrics()

    # Get tags for a specific metric
    tags = metrics['scores'].tags()  # e.g., ['rd.Tag.SCORE', 'rd.Tag.STATISTIC', 'rd.Tag.DISTRIBUTION']

    for metric in metrics.values_by_tag(rd.Tag.ALTERER):
        # ... access all metrics related to alterers (crossover, mutation) ...
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create the evolution engine
    
    let engine = GeneticEngine::builder()
        .codec(IntCodec::vector(10, 0..100))
        .minimizing()
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    // Run the engine
    let result = engine.run(|generation| generation.index() >= 1000);

    // Access the metrics from the result
    let metrics: MetricSet = result.metrics();

    // Get tags for a specific metric
    let tags = metrics.get("scores").unwrap().tags(); // [Tag::Score, Tag::Statistic, Tag::Distribution]
    for metric in metrics.iter_tagged(TagKind::Alterer) {
        // ... access all metrics related to alterers (crossover, mutation) ...
    }

    // Collect unique tags contained in the MetricSet
    let unique_tags = metrics.tags().collect::<Vec<_>>();
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