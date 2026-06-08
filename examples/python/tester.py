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


# codec = rd.IntCodec(shape=5, init_range=(0, 10))
# population = codec.population(size=20)


# def get_alleles(popualtion: rd.Population[int]) -> list[list[int]]:
#     chromosomes = [pheno.genotype()[0] for pheno in popualtion]
#     result = []
#     for chrom in chromosomes:
#         alleles = [gene.allele() for gene in chrom]
#         result.append(alleles)

#     return result


# before_alleles = get_alleles(population)

# mutator = rd.Mutate.arithmetic(0.5)
# mutated_population = mutator.alter(population)
# after_alleles = get_alleles(mutated_population)


# for before, after in zip(before_alleles, after_alleles):
#     print(f"before: {before}, after: {after}")


codec = rd.GraphCodec(
    shape=(3, 2), vertex=[rd.Op.add(), rd.Op.mul()], edge=rd.Op.weight()
)
population = codec.population(size=20)

mutator = rd.Mutate.graph(1.0, 1.0)
mutated_population = population

for _ in range(50):
    mutated_population = mutator.alter(mutated_population)

one = population[0].genotype()
two = mutated_population[0].genotype()

one = codec.decode(one)
two = codec.decode(two)


print(one)
print(two)
