#!/usr/bin/env python3
"""
NEAT evolution. This example shows how we can use radiate to evolve
graphs using the NEAT algorithm. Notice the only real difference in the
engine construction is the addition of the NEAT distance function and species
threshold in the `.diversity(...)` method.
"""

import radiate as rd

rd.random.seed(514)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])


@rd.novelty(
    distance=rd.Dist.euclidean(),
    k=15,
    threshold=0.1,
)
def behavior(genome: rd.Graph) -> list[float]:
    # For simplicity, we'll just use the number of nodes and edges as the behavior descriptor
    return genome.eval(inputs)[0]


engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .select(rd.Select.boltzmann(temp=4.0))
    .fitness(behavior)
    # .regression(inputs, answers, loss=rd.MSE)
    .diversity(
        rd.Dist.neat(excess=1.0, disjoint=1.0, weight_diff=3.0),
        species_threshold=0.15,
        target_species=5,
    )
    .alters(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.generations(100))
)

result = engine.run(ui=True)

eval_results = result.value().eval(inputs)
accuracy = rd.accuracy(result.value(), inputs, answers, loss=rd.MSE)

print(result)
print(result.metrics().dashboard())
print(accuracy)

for member in result.population():
    chromosome = member.genotype()[0]
    graph = rd.Graph.from_chromosome(chromosome)
    eval_results = graph.eval(inputs)
    accuracy = rd.accuracy(graph, inputs, answers, loss=rd.MSE)
    print(
        f"Member {member.id()} - Accuracy: {accuracy.accuracy():.4f} size: {len(graph)}"
    )
