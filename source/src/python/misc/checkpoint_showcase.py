# --8<-- [start:checkpoint]
import radiate as rd

target = "Hello, Radiate!"


def fitness_func(x: list[str]) -> int:
    return sum(1 for i in range(len(target)) if x[i] == target[i])


engine = (
    rd.Engine.char(len(target)).fitness(fitness_func).limit(rd.Limit.score(len(target)))
)

result = engine.run(checkpoint=(10, "checks", "pkl"))

# load from checkpoint from generation 10
engine = (
    rd.Engine.char(len(target))
    .fitness(fitness_func)
    .load_checkpoint("checks/chckpnt_10.pkl")
    .limit(rd.Limit.score(len(target)))
)

result_from_checkpoint = engine.run()
# --8<-- [end:checkpoint]
