# Events and Subscriptions

!!! warning ":construction: Under Construction :construction:"

    As of `12/15/2025`: These docs are a work in progress and may not be complete or fully accurate. Please check back later for updates.

Radiate provides an event system that allows you to monitor and react to the evolution process in real-time. This is great for:

- Tracking the progress of evolution
- Collecting metrics and statistics
- Implementing custom logging
- Visualizing the evolution process

## Overview

The event system in Radiate is built around the concept of event handlers or subscribers that can be attached to the `GeneticEngine`. These subscribers receive events at key points during the evolution process, allowing you to monitor and react to changes in the environment in real-time. The event system is designed to be flexible and extensible, allowing you to create custom event handlers that can perform various actions based on the evolution state.

The `GeneticEngine` trys it's best to off-load almost the entire compute workload of the subscribers (handlers) to the user - be aware of this when implementing your handlers.

!!! note "Threading Behavior"
    
    Currently, the rust implementation is multi-threaded, meaning if you have multiple subscribers, there is no guarantee of the order in which they will be called. For python, regardless of if you are using a free-threaded interpreter (3.13t/3.14t, ect) or not, the events will be dispatched on a single thread in the order they were added.

--- 
## Event Types

Radiate provides several key events that you can subscribe to. Here is the actual enum definition in Rust:

```rust
pub enum EngineEvent<T> {
    /// Triggered when the evolution process starts.
    /// Has no associated data, is simply a signal that evolution has begun.
    Start,
    /// Triggered when the evolution process stops. Provides the best individual, metrics, and score.
    Stop(T, MetricSet, Score),
    /// Triggered at the start of each epoch with the epoch index.
    EpochStart(usize),
    /// Triggered at the end of each epoch with the epoch index, best individual, metrics, and score.
    EpochComplete(usize, T, MetricSet, Score),
    /// Triggered when an improvement is found with the epoch index, best individual, and score.
    Improvement(usize, T, Score),
}
```

Below there is a brief description of each event type with its representative data structures expressed in json.

??? note "Start Event"

    This event is triggered when the evolution process starts. It provides an opportunity to initialize any resources or perform setup tasks before the evolution begins.

    ```json
    {
        'event_type': 'start_event'
    }
    ```

??? note "Stop Event"

    This event is triggered when the evolution process stops, either due to reaching a stopping condition or being manually stopped. It provides access to:

    - The final `metrics` of the evolution
    - The best individual found
    - The final `score`, or fitness, of the best individual

    ```json
    {
        'event_type': 'stop_event',
        // This will be a dictionary of metrics collected, see Engine's metrics docs for more info
        'metrics': ..., 
        // This will be the decoded best individual found so far. So, if you are 
        // evolving a vector of FloatGenes, this will be a list of floats
        'best': [3.9699993,  1.5489225, -1.7164116,  1.0756674, -1.932127 , -2.3247557], 
        'score': 0.3327971398830414
    }
    ```

??? note "Epoch Start Event"

    This event is triggered at the start of each generation (epoch) and provides the current generation number. It allows you to perform actions before the evolution step begins, such as resetting counters or logging initial state.

    ```json
    {
        'event_type': 'epoch_start_event',
        'index': 0  // Current generation number
    }
    ```

??? note "Epoch Complete Event"

    This event is triggered at the end of each generation (epoch) and provides information about:

    - The current generation number
    - The current `metrics` from the `GeneticEngine`
    - The best individual found from the `GeneticEngine` so far
    - The best `score`, or fitness, from the best individual

    ```json
    {
        'event_type': 'epoch_complete_event',
        'index': 0, // Current generation number
        // This will be the current metrics collected, see Engine's metrics docs for more info
        'metrics': ..., 
        // This will be the decoded best individual found so far. So, if you are 
        // evolving a vector of FloatGenes, this will be a list of floats
        'best': [3.9699993,  1.5489225, -1.7164116,  1.0756674, -1.932127 , -2.3247557], 
        'score': 0.3327971398830414
    }
    ```

??? note "Engine Improvement Event"

    This event is triggered when the engine finds a new best individual during the evolution process. It provides:

    - The index of the generation where the improvement occurred
    - The best individual found at that point
    - The `score`, or fitness, of the best individual

    ```json
    {
        'event_type': 'engine_improvement_event',
        'index': 0, // Current generation number
        // This will be the decoded best individual found so far. So, if you are 
        // evolving a vector of FloatGenes, this will be a list of floats
        'best': [3.9699993,  1.5489225, -1.7164116,  1.0756674, -1.932127 , -2.3247557], 
        'score': 0.3327971398830414
    }
    ```

---

## Subscribing to Events

You can subscribe to events in two ways:

### 1. Callback Function

The simplest way to subscribe to events is by providing a callback function:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        # Subscribe to all events using a lambda function
        subscribe=lambda event: print(event),  
        # ... other parameters ...
    )

    # or add it later
    engine.subscribe(lambda event: print(event))

    # Run the engine
    engine.run(rd.GenerationsLimit(100))
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mut engine = GeneticEngine::builder()
        .codec(your_codec)
        .fitness_fn(your_fitness_fn)
        .subscribe(|event: EngineEvent<Vec<f32>>| {
            if let EngineEvent::EpochComplete(index, best, metrics, score) = event {
                println!("Printing from event handler! [ {:?} ]: {:?}", index, score);
            }
        })
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 100
    });
    ``` 

### 2. Event Handler Class

For more complex event handling, you can create a custom event handler class:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Inherit from EventHandler, tell the super class which event you'd like to subscribe to, 
    # then override the on_event method
    class Subscriber(rd.EventHandler):
        def __init__(self):
            super().__init__(rd.EventType.EPOCH_COMPLETE)

        def on_event(self, event):
            print(f"Event: {event}")

    # Create an instance of your event handler
    handler = Subscriber()

    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        subscribe=handler,
        # ... other parameters ...
    )

    # or add it later
    engine.subscribe(handler)

    # Run the engine for 100 generations
    engine.run(rd.GenerationsLimit(100))
    ```

    Its also completely possible to create more advanced forms of visualization or logging through this method. For example, below we will collect the score distrubution from each epoch using polars then plot it with matplotlib.

    ```python
    class ScoreDistributionPlotter(rd.EventHandler):
        """
        Subscriber class to handle events and track metrics.
        We will use this to plot score distributions over generations then
        display the plot when the engine stops.
        """

        def __init__(self): 
            super().__init__() # By not passing an event type, we subscribe to all events
            self.history = []

        def on_event(self, event: rd.EngineEvent) -> None:
            if event.event_type() == rd.EventType.EPOCH_COMPLETE:
                ms = event.metrics().to_polars()
                epoch = event.index()
                ms = ms.with_columns(pl.lit(epoch).alias("epoch"))
                self.history.append(ms)
            elif event.event_type() == rd.EventType.STOP:
                df = pl.concat(self.history, how="diagonal_relaxed")
                plot_scores(df)


    def plot_scores(ms: pl.DataFrame):
        quant = (
            ms.filter((pl.col("name") == "scores") & (pl.col("kind") == "dist"))
            .select(
                "epoch",
                pl.col("min").alias("q0"),
                pl.col("mean").alias("q50"),
                pl.col("max").alias("q100"),
            )
            .sort("epoch")
        )

        pdf = quant.to_pandas()
        plt.figure(figsize=(8, 5))
        plt.fill_between(
            pdf["epoch"], pdf["q0"], pdf["q100"], alpha=0.2, label="min–max range"
        )
        plt.plot(pdf["epoch"], pdf["q50"], color="C0", linewidth=2, label="mean score")
        plt.xlabel("Epoch")
        plt.ylabel("Score")
        plt.title("Score distribution across generations")
        plt.legend()
        plt.tight_layout()
        plt.show()

    # Create an instance of your event handler
    handler = ScoreDistributionPlotter()

    engine = rd.GeneticEngine(
        codec=your_codec,
        fitness_func=your_fitness_func,
        subscribe=handler,
        # ... other parameters ...
    )

    # or add it later
    engine.subscribe(handler)

    # Run the engine for 100 generations
    engine.run(rd.GenerationsLimit(100))
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    struct MyHandler;

    impl EventHandler<Vec<f32>> for MyHandler {
        fn handle(&mut self, event: &EngineEvent<Vec<f32>>) {
            if let EngineEvent::EpochComplete(index, best, metrics, score) = event {
                println!("Printing from event handler! [ {:?} ]: {:?}", index, score);
            }
        }
    }

    // Create and configure the engine
    let mut engine = GeneticEngine::builder()
        .codec(your_codec)
        .subscribe(MyHandler)   // Add your handler here
        .fitness_fn(your_fitness_fn)
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 100
    });
    ``` 

## Best Practices

1. **Keep Event Handlers Light**:
    - Event handlers are called frequently during evolution
    - Avoid heavy computations in event handlers

2. **Use Multiple Subscribers**:
    - You can subscribe multiple handlers to the same engine
    - Separate concerns into different handlers
        - Example: one for logging, one for metrics, one for visualization

3. **Handle Errors Gracefully**:
    - Event handlers should not crash the evolution process
    - Log errors instead of raising exceptions - do not expect the `GeneticEngine` to throw exceptions

4. **Monitor Performance**:
    - Be aware that event handling adds some overhead depending on your implementation
    - Use built in `metrics` to track certain metrics or performance characteristics if possible
    - Be cautious of your implementation - consider disabling event handling in production if not essential

