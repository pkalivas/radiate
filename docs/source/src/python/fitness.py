# --8<-- [start:rastrigin]
import radiate as rd
import math

A = 10.0
RANGE = 5.12
N_GENES = 2


def fit(x: list[float]) -> float:
    value = A * N_GENES
    for i in range(N_GENES):
        value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
    return value


codec = rd.FloatCodec(N_GENES, init_range=(-RANGE, RANGE))
engine = rd.Engine(codec).fitness(fit).minimizing()
# --8<-- [end:rastrigin]

# --8<-- [start:fitness_decorator]
import radiate as rd


@rd.fitness  # <- this decorator
def fit_decorated(x: list[float]) -> float:
    value = A * N_GENES
    for i in range(N_GENES):
        value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
    return value


# ... fill in the rest of your engine setup as normal (check first example above)...
# --8<-- [end:fitness_decorator]

# --8<-- [start:batch_fitness]
import radiate as rd
import math

A = 10.0
RANGE = 5.12
N_GENES = 2


# NOTE this function expects a batch of inputs and returns a batch of outputs
# the order in which the inputs are given is the order in which the outputs are returned
def fit_batch(x: list[list[float]]) -> list[float]:
    assert len(x) > 1

    results = []
    for member in x:
        value = A * N_GENES
        for i in range(N_GENES):
            value += member[i] ** 2 - A * math.cos(
                (2.0 * 3.141592653589793 * member[i])
            )
        results.append(value)
    return results


codec = rd.FloatCodec(N_GENES, init_range=(-RANGE, RANGE))

# Create the genetic engine with batch fitness function.
# Just wrap your fitness function in 'rd.BatchFitness'
engine = rd.Engine(codec).fitness(rd.BatchFitness(fit_batch)).minimizing()
# --8<-- [end:batch_fitness]

# --8<-- [start:batch_fitness_decorator]
import radiate as rd
import math

A = 10.0
RANGE = 5.12
N_GENES = 2


# NOTE this function expects a batch of inputs and returns a batch of outputs
# the order in which the inputs are given is the order in which the outputs are returned
@rd.fitness(batch=True)  # <- this decorator with batch=True
def fit_batch_decorated(x: list[list[float]]) -> list[float]:
    assert len(x) > 1

    results = []
    for member in x:
        value = A * N_GENES
        for i in range(N_GENES):
            value += member[i] ** 2 - A * math.cos(
                (2.0 * 3.141592653589793 * member[i])
            )
        results.append(value)
    return results


codec = rd.FloatCodec(N_GENES, init_range=(-RANGE, RANGE))

# Create the genetic engine with batch fitness function.
# NOTE: We no longer need to wrap 'fitness_fn' in 'rd.BatchFitness'
engine = rd.Engine(codec).fitness(fit_batch_decorated).minimizing()
# --8<-- [end:batch_fitness_decorator]

# --8<-- [start:novelty_search]
import radiate as rd


# Define a behavioral descriptor
def behavior(individual: list[float]) -> list[float]:
    # Return behavioral characteristics
    # Some code that describes the behavior of a vector
    # The individual here is a list[float] because we are using a FloatCodec vector below
    ...


# Create novelty search fitness function
novelty_fitness = rd.NoveltySearch(
    descriptor=behavior,
    # can use any of the distance inputs. The engine will use this to
    # determine how 'novel' an individual is compared to the other's in the
    # archinve or population, ultimently resulting in the individuals fitness score.
    distance=rd.CosineDistance(),  # Distance metric to use,
    k=10,  # Number of nearest neighbors to consider
    threshold=0.1,  # Novelty threshold for archive addition
    archive_size=1000,  # defaults to 1000
)

engine = (
    # The decoded value of your codec (a list[float] in this case) will be fed into the `behavior` function of your NoveltySearch fitness_func
    rd.Engine.float(10, init_range=(0, 10), bounds=(0, 10))
    .fitness(novelty_fitness)
    .maximizing()  # We want to maximize novelty - however this is the default so its not necessary to define
)
# --8<-- [end:novelty_search]

# --8<-- [start:novelty_decorator]
import radiate as rd


# Define a behavioral descriptor
@rd.novelty(distance=rd.Dist.cosine(), k=10, threshold=0.1, archive=1000)
def behavior_decorated(individual: list[float]) -> list[float]:
    # Return behavioral characteristics
    # Some code that describes the behavior of a vector
    # The individual here is a list[float] because we are using a FloatCodec vector below
    ...


engine = rd.Engine.float(10, init_range=(0, 10)).fitness(behavior_decorated)
# --8<-- [end:novelty_decorator]
