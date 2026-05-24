import radiate as rd
import polars as pl
import matplotlib.pyplot as plt


# Setup (not shown): stand-in codec + fitness for the snippets below.
your_codec = rd.FloatCodec(2, init_range=(0.0, 1.0))


def your_fitness_func(x):
    return sum(x)


# --8<-- [start:lambda_subscribe]
import radiate as rd

engine = (
    rd.Engine(your_codec)
    .fitness(your_fitness_func)
    .subscribe(lambda event: print(event))  # Subscribe to all events using a lambda function
    # ... other parameters ...
)

# Run the engine for 100 generations
engine.run(rd.Limit.generations(100))
# --8<-- [end:lambda_subscribe]

# --8<-- [start:handler_subclass]
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

engine = rd.Engine(
    codec=your_codec,
    fitness_func=your_fitness_func,
    subscribe=handler,
    # ... other parameters ...
)

# or add it later
engine.subscribe(handler)

# Run the engine for 100 generations
engine.run(rd.GenerationsLimit(100))
# --8<-- [end:handler_subclass]

# --8<-- [start:score_plotter]
class ScorePlotterHandler(rd.EventHandler):
    """
    An event handler that collects best scores over epochs and plots them at the end.
    1. On EPOCH_COMPLETE, it appends the best score to a list.
    2. On STOP, it creates a DataFrame and plots the scores over generations.
    """

    def __init__(self):
        super().__init__()  # Not specifying an event type to listen to all events
        self.scores = []

    def on_event(self, event: rd.EngineEvent) -> None:
        if event.event_type() == rd.EventType.EPOCH_COMPLETE:
            best_score = event.score()
            self.scores.append(best_score)
        elif event.event_type() == rd.EventType.STOP:
            df = pl.DataFrame(
                {"Generation": list(range(len(self.scores))), "Score": self.scores}
            )
            plt.plot(df["Generation"], df["Score"])
            plt.xlabel("Generation")
            plt.ylabel("Best Score")
            plt.title("Best Score over Generations")
            plt.grid(True)
            plt.show()


# Create an instance of your event handler
handler = ScorePlotterHandler()

engine = (
    rd.Engine(codec=your_codec)
    .fitness(your_fitness_func)
    .subscribe(handler)  # Add your handler here
    # ... other parameters ...
)

# Run the engine for 100 generations
engine.run(rd.GenerationsLimit(100))
# --8<-- [end:score_plotter]

# --8<-- [start:metric_collector]
import radiate as rd

# Create an instance of the MetricCollector
collector = rd.MetricCollector()

engine = (
    rd.Engine.float(2, init_range=(0.0, 1.0))  # configure your engine as normal
    .fitness(your_fitness_func)
    .subscribe(collector)  # Subscribe the MetricCollector to the engine
    # ... other parameters ...
)

# Run the engine for 100 generations
engine.run(rd.Limit.generations(100))

# After the run, you can access the collected metrics
# Convert collected metric sets to a df where each row is a single metric (includes all collected metrics).
df = collector.to_polars(lazy=False)  # optional lazy arg - defaults to False

# Same as above but with pandas instead of polars
df = collector.to_pandas()

# Plot specific metrics to a matplotlib line plot
collector.plot("scores.best", "rate.diversity")
# --8<-- [end:metric_collector]
