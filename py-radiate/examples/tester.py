import radiate as rd


gene = rd.Gene.float(init_range=(-5.0, 5.0))

print(gene)


def fit(val) -> float:
    print(val)
    return sum(val)


# engine = rd.Engine.float(genes=gene).fitness(fit).minimizing()

# result = engine.run(rd.Limit.generations(10))

# print(result)


chromosome = rd.Chromosome.float(5, init_range=(-5.0, 5.0))

print(chromosome)

engine = (
    rd.Engine.float(chromosomes=chromosome, use_numpy=True)
    .fitness(fit)
    .minimizing()
    .limit(rd.Limit.generations(10))
)

result = engine.run()
