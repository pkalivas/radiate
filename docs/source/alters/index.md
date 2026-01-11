
# Alterers

Alterers are genetic operators that modify the genetic material of individuals in a `population`. In Radiate, there are two main types of alterers:

1. **Mutators**: Operators that modify individual genes or chromosomes
2. **Crossovers**: Operators that combine genetic material from two parents to create offspring

These operators modify the `population` and are essential for the genetic algorithm to explore the search space effectively. As such, the choice of `alterer` can have a significant impact on the performance of the genetic algorithm, so it is important to choose an `alterer` that is well-suited to the problem being solved.

---

## Best Practices

1. **Rate Selection**:
    - Start with conservative rates (0.01 for mutation, 0.5-0.8 for crossover)
    - Adjust based on problem characteristics
    - Higher rates increase exploration but may disrupt good solutions

2. **Choosing the Right Alterer**:
    - For continuous problems: Use Gaussian or Arithmetic mutators with Blend/Intermediate crossover
    - For permutation problems: Use Swap/Scramble mutators with PMX or Shuffle crossover
    - For binary problems: Use Uniform mutator with Multi-point or Uniform crossover

3. **Combining Alterers**:
    - It's often beneficial to use multiple alterers
    - Example: Combine a local search mutator (Gaussian) with a global search crossover (Multi-point)
    - Monitor population diversity to ensure proper balance

4. **Parameter Tuning**:
    - Start with default parameters
    - Adjust based on problem size and complexity
    - Use smaller rates for larger problems

## Common Pitfalls

1. **Too High Mutation Rates**:
    - Can lead to random search behavior
    - May destroy good solutions before they can be exploited
    - Solution: Start with low rates (0.01-0.1) and adjust based on results

2. **Inappropriate Crossover Selection**:
    - Using permutation crossovers for continuous problems
    - Using continuous crossovers for permutation problems
    - Solution: Match the crossover type to your problem domain

3. **Ignoring Problem Constraints**:
    - Some alterers may produce invalid solutions
    - Solution: Use appropriate alterers or implement repair mechanisms

4. **Poor Parameter Tuning**:
    - Using the same parameters for all problems
    - Solution: Experiment with different parameters and monitor performance
