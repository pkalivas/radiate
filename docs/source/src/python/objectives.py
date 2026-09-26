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
