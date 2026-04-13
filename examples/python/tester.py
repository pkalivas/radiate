from pprint import pprint

import radiate as rd

gene = rd.Gene.int(init_range=(0, 10))
other = rd.Gene.int(init_range=(0, 10))

print(gene)


def fit(val) -> float:
    print(val)
    return sum(val)


engine = rd.Engine.int(genes=[gene, other]).fitness(fit).minimizing()

result = engine.run(rd.Limit.generations(10))
print(result)

chromosome = rd.Chromosome.int(5, init_range=(0, 10))


engine = (
    rd.Engine.int(chromosomes=chromosome, use_numpy=True)
    .fitness(fit)
    .minimizing()
    .limit(rd.Limit.generations(10))
)

result = engine.run()
print(result.metrics().dashboard())

metrics = result.metrics()
expr = rd.metric("scores").mean() == rd.metric("scores").min()

print(type(expr))
print(metrics.project(expr))
print(metrics.project(expr))
print(metrics.project(expr))
print(metrics.project(expr))


metrics = rd.MetricSet(
    one=list(range(10)),
    two=list(range(10, 20)),
)

# for metric in metrics.values():
#     pprint(metric.to_dict())

expr = rd.metric("one").min() < -1.0
other = (
    rd.when(rd.metric("one").min() < -1.0)
    .then(rd.metric("two").mean())
    .otherwise(123123)
)

t = rd.every(2).then(rd.element().rolling(10).mean()).otherwise(rd.lit(-1.0))

for i in range(10):
    print(t.apply(float(i)))

# print(metrics.project(other))
# print(metrics.project(expr))
# metrics.upsert("one", -10.0)
# print(metrics.project(other))
# print(metrics.project(expr))

# pprint(metrics["one"].to_dict())

# pprint(other)

# gene = rd.Gene.float(init_range=(-5.0, 5.0))

# print(gene)


# def fit(val) -> float:
#     print(val)
#     return sum(val)


# # engine = rd.Engine.float(genes=gene).fitness(fit).minimizing()

# # result = engine.run(rd.Limit.generations(10))

# # print(result)


# chromosome = rd.Chromosome.float(5, init_range=(-5.0, 5.0))

# print(chromosome)

# engine = (
#     rd.Engine.float(chromosomes=chromosome, use_numpy=True)
#     .fitness(fit)
#     .minimizing()
#     .limit(rd.Limit.generations(10))
# )

# result = engine.run()
