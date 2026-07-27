import radiate as rd


def my_fitness_fn(x):
    return 0.001


# --8<-- [start:applying]
import radiate as rd

# A rate can be a plain float (the common case — it's wrapped into a constant
# `Expr` automatically) or a full `Expr` for something metric-driven.
engine = (
    rd.Engine.float(10, init_range=(-5.0, 5.0))
    .fitness(my_fitness_fn)
    .minimizing()
    .alters(
        rd.Mutate.gaussian(rate=0.1),
        rd.Cross.blend(rate=0.5),
    )
)
# --8<-- [end:applying]

# --8<-- [start:constant]
import radiate as rd

mutator = rd.Mutate.gaussian(rate=0.1)
# --8<-- [end:constant]

# --8<-- [start:linear_ramp]
import radiate as rd

# Ramp from 0.5 down to 0.05 over the first 50 generations, then hold at 0.05
start, end, duration = 0.5, 0.05, 50
progress = (rd.Expr.generation() / duration).clamp(0.0, 1.0)
linear_ramp = rd.Expr.lit(start) + (rd.Expr.lit(end) - start) * progress
# --8<-- [end:linear_ramp]

# --8<-- [start:stepped]
import radiate as rd

# Jump to a new rate at fixed generation boundaries
gen = rd.Expr.generation()
stepped = (
    rd.Expr.when(gen < 25)
    .then(0.1)
    .otherwise(rd.Expr.when(gen < 75).then(0.5).otherwise(0.9))
)
# --8<-- [end:stepped]

# --8<-- [start:periodic]
import radiate as rd

# Oscillate between high and low exploration every 10 generations
periodic = rd.Expr.every(10).then(0.9).otherwise(0.1)
# --8<-- [end:periodic]

# --8<-- [start:exponential_decay]
import radiate as rd

# Half-life decay: starts at `start`, halves every `half_life` generations,
# settling toward `end`
start, end, half_life = 0.5, 0.05, 25
decay = rd.Expr.lit(0.5) ** (rd.Expr.generation() / half_life)
exponential_decay = rd.Expr.lit(end) + (rd.Expr.lit(start) - end) * decay
# --8<-- [end:exponential_decay]

# --8<-- [start:metric_driven]
import radiate as rd

# Boost mutation once the best score has stagnated for 20 generations
stagnation_boost = (
    rd.Expr.when(rd.Expr.is_stagnant("scores.best", patience=20))
    .then(0.30)
    .otherwise(0.05)
)

# Or track a continuous signal: dial mutation down as scores stabilize
volatility_driven = (
    rd.Expr.select("score.volatility").rolling(20).mean().clamp(0.01, 0.5)
)
# --8<-- [end:metric_driven]
