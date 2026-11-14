import radiate as rd

# Setup (not shown): a tiny regression dataset so the showcase runs standalone.
# This file is a long-running, plotting end-to-end demo — it is intentionally excluded
# from the fast snippet test suite (see py-radiate/tests/docs/test_user_guide.py), but its
# API is exercised by the focused expressions.py snippets and its include is build-checked.
inputs = [[x / 10.0] for x in range(-10, 11)]
answers = [[4.0 * v[0] ** 3 - 3.0 * v[0] ** 2 + v[0]] for v in inputs]

# --8<-- [start:example]
import radiate as rd

rd.random.seed(90)

# Define a few parameters we'll need
# 1. Define the target number of species we want (4)
target_species = 4
# 2. Define a few constants for scaling the signals - remember this is a feedback loop
KP = 0.05  # Proportional gain
KI = 0.005  # Integral gain
KD = 0.02  # Derivative gain
# 3. Define an initial threshold value that we will start at
initial_threshold = 0.5

# Now, lets build our expression
# 1. We want an error signal to say how far we are from our desired target
raw_error = rd.Expr.select("species.count").error(target_species)
# 2. Now we need a proportional signal based on the raw error, smoothed by the proportional gain
proportional = (
    rd.Expr.select("species.count").rolling(3).mean().error(target_species) * KP
)

# 3. Next, we need to define our integral and derivative signals for the PID controller
# Integral: accumulated recent error over a rolling window
# Derivative: velocity of the error — anticipates rising/falling count
integral = raw_error.rolling(20).sum() * KI
derivative = raw_error.rolling(5).slope() * KD

# 4. Finally, combine the proportional, integral, and derivative signals to form the PID control signal.
# However, we need to make sure that the engine _has_ values in it's metric set to use (again, this is a feedback loop),
# so we want to wait at least one generation before our PID control signal is applied
pid_control_signal = (
    rd.Expr.when(rd.Expr.select("index") < 2)
    .then(initial_threshold)  # Use the initial threshold for the first generation
    .otherwise(
        rd.Expr.select("Our.PID") + proportional + integral + derivative
    )  # Notice the feedback! (the PID control signal depends on its own previous value)
    .clamp(0.0, target_species * 2.5)
    .alias("Our.PID")  # Give our PID control signal a name in the metric set
)

# Track a rolling 25 generation mean of the species count so we can plot it against the raw count
species_count_mean = rd.Expr.select("species.count").rolling(25).mean()

collector = rd.MetricCollector()

# Now, lets plug in our PID control signal into the engine's species threshold!
engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.sum()],
        edge=rd.Op.weight(),
        output=rd.Op.sum(),
    )
    .regression(inputs, answers, loss=rd.MSE)
    .subscribe(collector)
    .diversity(rd.Dist.neat(1.0, 1.0, 3.0), threshold=pid_control_signal)
    .metrics(species_count_mean=species_count_mean)
    .alter(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)

result = engine.run(log=True)

collector.plot_titled(
    "Species Count vs Rolling 25gen Mean Species Count",
    ["species_count_mean", "species.count"],
)
# --8<-- [end:example]

# --8<-- [start:all_else_same]
engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.sum()],
        edge=rd.Op.weight(),
        output=rd.Op.sum(),
    )
    .regression(inputs, answers, loss=rd.MSE)
    .diversity(
        rd.Dist.neat(1.0, 1.0, 3.0), target=4
    )  # <- Set the target number of species to 4
    .alter(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
)
# --8<-- [end:all_else_same]
