import radiate as rd

# metrics = rd.MetricSet(one=list(range(10)), two=list(range(10, 20)), three=4)
# metrics.upsert("four", 2)
# metrics.upsert("four", 3)
# metrics.upsert("four", 4)
# metrics.upsert("four", 5)


# print(metrics.dashboard())


def fit(indv: list[int]) -> float:
    return sum(indv)


engine = rd.Engine.int(5).fitness(fit).minimizing().limit(rd.Limit.generations(10))

for epoch in engine:
    print(epoch.index())


# metrics = rd.MetricSet()

# base = rd.Expr.select("test")

# rolling = base.rolling(2)
# first_rolling = rolling.last()
# combined = base * first_rolling

# for i in range(10):
#     metrics.upsert("test", i)
#     print(f"combined: {combined.eval(metrics)}")
