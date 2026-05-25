
# Diversity

Diversity is an opt-in aspect of `radiate`'s genetic algorithm. At its core, this operator helps maintain a healthy population and prevent premature convergence. By using this operator, the `GeneticEngine` will split the `population` up into `species` by measuring the genetic distance, or diversity, between individuals during the evolution process. Its important to note that adding a diversity operator will increase the computational cost of the algorithm, so it should be used judiciously based on the problem at hand.

---

Diversity in `radiate` is implemented through two main components:

1. **Species Management**: Mechanisms to group similar individuals and maintain population diversity
2. **Diversity Measurement**: Methods to quantify how different individuals are from each other. This is typically done by measuring the genetic distance between individuals - meaning the actual `chromosomes` and `genes` that make up the individuals.
