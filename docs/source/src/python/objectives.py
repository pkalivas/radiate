import radiate as rd


# Setup (not shown): stand-in objective functions for the multi-objective snippets.
def obj1_fitness_func(x):
    return sum(x)


def obj2_fitness_func(x):
    return sum((v - 0.5) ** 2 for v in x)


# --8<-- [start:minimizing]
import radiate as rd

# Create an engine that has a genome of 1 chromosome with 10 float genes, initialized & bound between 0.0 and 1.0
engine = (
    rd.Engine.float(10, init_range=(0.0, 1.0))  # Example codec
    .fitness(lambda x: sum(x))  # return a value to minimize
    .minimizing()  # Configure for minimization
    # ... other parameters ...
)
# --8<-- [end:minimizing]

# --8<-- [start:maximizing]
import radiate as rd

# Create an engine that has a genome of 1 chromosome with 10 float genes, initialized & bound between 0.0 and 1.0
# Note that maximization is the default, so you could omit the '.maximizing()' call and it would still maximize
engine = (
    rd.Engine.float(10, init_range=(0.0, 1.0))
    .fitness(lambda x: sum(x))  # return a value to maximize
    .maximizing()  # Configure for maximization
    # ... other parameters ...
)
# --8<-- [end:maximizing]

# --8<-- [start:multi_objective]
import radiate as rd

# Create an engine that has a genome of 1 chromosome with 10 float genes, initialized & bound between 0.0 and 1.0
engine = (
    rd.Engine.float(10, init_range=(0.0, 1.0))
    .fitness(
        lambda x: [obj1_fitness_func(x), obj2_fitness_func(x)]
    )  # Return list of objectives
    .objective(rd.MIN, rd.MAX)  # Minimize obj1, maximize obj2
    .front_range(800, 900)  # Pareto front size range
    # ... other parameters ...
)
# --8<-- [end:multi_objective]

# --8<-- [start:multi_objective_selectors]
import radiate as rd

engine = (
    rd.Engine.float(10, init_range=(0.0, 1.0))  # Example codec
    .fitness(
        lambda x: [obj1_fitness_func(x), obj2_fitness_func(x)]
    )  # Return list of objectives
    .objective(rd.MIN, rd.MAX)  # Minimize obj1, maximize obj2
    .front_range(800, 900)  # Pareto front size range
    .select(
        offspring=rd.Select.tournament_nsga2(),  # crowded-comparison tournament for parents
        survivor=rd.Select.nsga2(),  # rank + crowding distance for survivors
        frac=0.5,  # keep more of the already-evaluated front each generation
    )  # Set MO selectors
    .alter(
        rd.Cross.sbx(0.8, 20.0),  # Simulated Binary Crossover
        rd.Mutate.polynomial(0.1, 20.0),  # Polynomial Mutation
    )
    # ... other parameters ...
)
# --8<-- [end:multi_objective_selectors]

# --8<-- [start:front_reading]
import radiate as rd

codec = rd.FloatCodec(shape=10, init_range=(0.0, 1.0))

engine = (
    rd.Engine(codec)
    .fitness(lambda x: [obj1_fitness_func(x), obj2_fitness_func(x)])
    .objective(rd.MIN, rd.MIN)  # conflicting: obj1 pulls genes to 0.0, obj2 to 0.5
    .front_range(800, 900)
    .limit(rd.Limit.generations(200))
)

result = engine.run()

# The archive of the best trade-offs found over the whole run
front = result.front
print(f"{len(front)} solutions on the front")

for member in front:
    scores = member.score  # one value per objective
    params = codec.decode(member.genotype())  # back to the problem's representation
# --8<-- [end:front_reading]

# --8<-- [start:front_analysis]
# Reference point: worse than every member on every objective
hypervolume = front.hypervolume([10.0, 3.0])
entropy = front.entropy()
crowding = front.crowding_distance()

# Drop the 2% most isolated members
removed = front.remove_outliers(0.02)
# --8<-- [end:front_analysis]
