import radiate as rd


def my_fitness_fn(x):
    return x


# --8<-- [start:minimal_engine]
import radiate as rd

# Only a codec and a fitness function are required - everything else uses its default.
engine = (
    rd.Engine.float(init_range=(0.0, 1.0))
    .fitness(my_fitness_fn)
    .limit(rd.Limit.generations(100))
)

result = engine.run()
# --8<-- [end:minimal_engine]
