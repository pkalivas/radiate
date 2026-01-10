# Metrics

Metric collection in radiate is interwoven into every aspect of the evolutionary process. It uses the [Kahan summation algorithm](https://en.wikipedia.org/wiki/Kahan_summation_algorithm) paired with [Welford's one-pass online algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm) for fast, accurate, and numerically stable computation of statistics. All of this combined provides robust and reliable metric tracking throughout the evolutionary process. Using the `MetricSet` (a collection of independent `Metric`s) we can collect a whole host statistics that span the entire evolutionary process allowing us to gain deep insights into the evolutionary dynamics.

---

## MetricSet

The `MetricSet` is an object (struct) provided to the user in two main forms:

1. On the engine's `Generation` - given to the user after each epoch or each pass of the evolution process.
2. Through the engine's eventing system. Various events emit metric data allowing the user to track metrics or derive their own in real-time.

Each `metric` can include one or both of the following statistical types:

1. **Statistic** - for general numerical data

    The `Statistic` exposes a number of different statistical measures that can be used to summarize the data, such as, `last_value`, `count`, `min`, `max`, `mean`, `sum`, `variance`, `std_dev`, `skewness`, and `kurtosis`. 

2. **TimeStatistic** - for time-based data

    Similarly, the `TimeStatistic` exposes the same measures, however the data is assumed to be time-based. As such, the results are expressed as a `Duration::from_secs_f32(value)`.

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
| `replace_age`      | The number of individuals replaced based on age. |
| `replace_invalid`  | The number of individuals replaced based on invalid structure (e.g. Bounds) |
| `genome_size`      | The size of each genome over the evolution process. This is usually static and doesn't change. |
| `unique_members`   | The number of unique members in the `Ecosystem`. |
| `unique_scores`    | The number of unique scores in the `Ecosystem`. |
| `new_children`     | The number of new children created each generation through either mutation or crossover (or both). |
| `survivor_count`   | The number of individuals that survived to the next generation - summation throughout the evolution process. |
| `carryover_rate`   | The rate at which unique individuals are carried over to the next generation - `survivor_count` per generation / population size. |
| `evaluation_count` | The total number of evaluations performed per generation. |
| `diversity_ratio`  | The ratio of unique scores to the size of the `Ecosystem`. |
| `score_volatility` | The volatility of the scores in the `Ecosystem`. This is calculated as the standard deviation of the scores / mean. |
| `best_score_improvement` | The improvement of the best score from the previous generation to the current generation - either a 1 or 0 each generation. |

## Multi-objective Metrics

Additional metrics collected when using multi-objective optimization:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `front_additions`            | The number of members added to the Pareto front each generation. | 
| `front_entropy`  | The entropy of the Pareto front throughout the evolution process - only calculated every 10 generations (its kinda an expensive calculation). |
| `front_removals`  | The number of members removed from the Pareto front each generation. |
| `front_comparisons`  | The number of comparisons made to update the Pareto front each generation. |
| `front_size`  | The size of the Pareto front each generation. |   
| `front_filters`  | The number of times the Pareto front was filtered each generation. |

## Species-based Metrics

Additional metrics collected when using species-based diversity:

| Name                | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| `species_count`    | The number of `species` in the 'Ecosystem`. |
| `species_removed`  | The number of `species` removed based on stagnation. |
| `species_created`  | The number of `species` created in the `Ecosystem`. |
| `species_died`     | The number of `species` that have died in the `Ecosystem`. |
| `species_age`      | The age of all the `species` in the `Ecosystem`. |
| `species_age_fail` | The count of species that have failed based on age each generation. |
| `species_eveness` | The evenness of the species distribution in the `Ecosystem`. |
| `largest_species_share` | The share of the largest species in the `Ecosystem`. |
| `species_new_ratio` | The ratio of new species created each generation. |


## Accessing Metrics

These can be accessed through the `metrics()` method of the epoch, which returns a `MetricSet`. 

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create an engine
    engine = rd.GeneticEngine(
        codec=rd.FloatCodec.scalar(0.0, 1.0), 
        fitness_fn=my_fitness_fn,  # Single objective fitness function
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
    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_function,
        # ... other parameters ...
    )

    # Run the engine
    result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(1000)])

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
