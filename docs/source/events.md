# Events and Subscriptions

Radiate provides an event system that allows you to monitor and react to the evolution process in real-time. This is great for:

- Tracking the progress of evolution
- Collecting metrics and statistics
- Implementing custom logging or logic based on the state of evolution
- Visualizing the evolution process

## Overview

The event system in `radiate` is built around the concept of event handlers or subscribers that can be attached to the `GeneticEngine`. These subscribers receive events at key points during the evolution process, allowing you to monitor and react to changes in the environment in real-time. The event system is designed to be flexible and extensible, allowing you to create custom event handlers that can perform various actions based on the evolution state.

The `GeneticEngine` offloads nearly all of a subscriber's compute cost onto the handler itself — so be mindful of this when implementing your handlers; expensive work here _can_ slow the whole run.

!!! note "Threading Behavior"
    
    Currently, the rust implementation is multi-threaded (if multi-threaded [executors](executors.md) are used), meaning if you have multiple subscribers, there is no guarantee of the order in which they will be called. For python, regardless of if you are using a free-threaded interpreter (3.13t/3.14t, etc) or not, the events will be dispatched on a single thread in the order they were added.

--- 
## Event Types

Radiate provides several key events that you can subscribe to. The variants live on the `EngineEventInner<T>` enum; in a handler you receive an `EngineEvent<T>` — a cheap, clonable `Arc` wrapper around it — and pattern-match against the inner enum via `event.inner()`:

```rust
pub enum EngineEventInner<T> {
    /// Triggered when the evolution process starts.
    /// Has no associated data, is simply a signal that evolution has begun.
    Start,
    /// Triggered when the evolution process stops. Provides the best individual, metrics, and score.
    Stop(usize, T, MetricSet, Score),
    /// Triggered at the start of each epoch with the epoch index.
    EpochStart(usize),
    /// Triggered at the end of each epoch with the epoch index, best individual, metrics, and score.
    EpochComplete(usize, T, MetricSet, Score, Objective),
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
        'index': 0, // Current generation number
        // This will be a MetricSet of metrics collected, see Engine's metrics docs for more info
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
        'score': 0.3327971398830414,
        'objective': ['min']  // The optimization objective(s) used in this run
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

### Callback Function

The simplest way to subscribe to events is by providing a callback function:

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/events.py:lambda_subscribe"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/events.rs:callback"
    ```

### Event Handler Class

For more complex event handling, you can create a custom event handler class:

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/events.py:handler_subclass"
    ```

    It's also completely possible to create more advanced forms of visualization or logging through this method. For example, below we will collect the scores from each epoch then use polars to create a DataFrame and finally plot it with matplotlib.

    ```python
    --8<-- "python/events.py:score_plotter"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/events.rs:handler"
    ```


## Built in Handlers

=== ":fontawesome-brands-python: Python"

    As of `4/25/2026`, the python implementation includes one built in event handler called the `MetricCollector`. This handler collects the [metric set](engine/metrics.md) at the end of each epoch and stores it in a list for later use. Note to use this handler to its fullest capacity, you should install radiate with the `polars` (or `pandas`) and `matplotlib` extras, as shown below:

    ```bash
    uv add "radiate[polars,pandas,matplotlib]"
    ```

    You can use this handler as follows (this is great when using radiate inside a `.ipynb` notebook):

    ```python
    --8<-- "python/events.py:metric_collector"
    ```

=== ":fontawesome-brands-rust: Rust"

    No built-in handlers in rust yet.

---

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

