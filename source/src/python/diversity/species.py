import radiate as rd


def your_fitness_func(x):
    return sum(abs(v) for v in x)


def your_char_fit_func(x):
    return sum(ord(c) for c in x)


# --8<-- [start:threshold]
import radiate as rd

engine = (
    rd.Engine.char(10)
    .fitness(your_char_fit_func)
    # A distance measure turns speciation on; the threshold sets how close
    # two individuals must be (per the measure) to share a species.
    .diversity(rd.Dist.hamming(), threshold=0.5)
)
# --8<-- [end:threshold]

# --8<-- [start:dynamic_threshold]
import radiate as rd

# `threshold` accepts a float or an `Expr` — the same mechanism covered in
# depth on the Rates page. Here it widens from 0.3 to 0.9 across the first
# 100 generations: start fine-grained (many small species), then coarsen to
# encourage convergence.
start, end, duration = 0.3, 0.9, 100
progress = (rd.Expr.generation() / duration).clamp(0.0, 1.0)
widening_threshold = rd.Expr.lit(start) + (rd.Expr.lit(end) - start) * progress

engine = (
    rd.Engine.float(2)
    .fitness(your_fitness_func)
    .diversity(rd.Dist.euclidean(), threshold=widening_threshold)
)
# --8<-- [end:dynamic_threshold]

# --8<-- [start:age]
import radiate as rd

engine = (
    rd.Engine.float(2)
    .fitness(your_fitness_func)
    .diversity(rd.Dist.euclidean(), threshold=0.5)
    # A species that survives this many generations without improving its best
    # score is culled, and its members sit out crossover/mutation that generation.
    .age(max_species_age=25)
)
# --8<-- [end:age]

# --8<-- [start:target_species_count]
import radiate as rd

# `target_species` is an alternative to `species_threshold` that tries to maintain a certain number of
# species. The engine will adjust the threshold up or down as needed to try to meet the target count.
engine = (
    rd.Engine.float(2)
    .fitness(your_fitness_func)
    .diversity(rd.Dist.euclidean(), target=4)
)

# --- Below is what happens internally in the engine, you do not need to do this ---

# `target` builds this exact `Expr` and passes it to `threshold` under the
# hood — a small PID-style controller that nudges the threshold based on the
# error between the current and target species count:
target_species = 4
base_val = 0.5  # the initial species_threshold

raw_error = rd.Expr.select("species.count").error(target_species)

# Proportional: smoothed count so single-generation bursts don't cause hard jumps
proportional = (
    rd.Expr.select("species.count").rolling(3).mean().error(target_species) * 0.05
)

# Integral: accumulated recent error. Derivative: velocity of the error,
# anticipating a rising/falling count before it overshoots.
integral = raw_error.rolling(20).sum() * 0.005
derivative = raw_error.rolling(5).slope() * 0.02

species_threshold = (
    rd.Expr.when(rd.Expr.generation() < 2)
    .then(base_val)
    .otherwise(
        rd.Expr.select("species.threshold") + proportional + integral + derivative
    )
    .clamp(0.0, target_species * 2.5)
)
# --8<-- [end:target_species_count]
