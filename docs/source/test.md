# Selection Strategies

??? abstract "What is Selection?"
    Selection is a crucial part of genetic algorithms that determines which individuals from the current population will:
    1. Survive directly to the next generation
    2. Be chosen as parents to create new offspring through genetic operations (mutation and crossover)

    Think of selection like natural selection in nature - it's how we decide which solutions are "fit enough" to pass their traits to the next generation.

## Selection Types

Radiate provides several selection strategies, each with its own characteristics and use cases. The choice of selector can significantly impact your algorithm's performance.

### 1. Tournament Selection

??? tip "Best for: Maintaining diversity while favoring good solutions"
    Tournament selection is like holding a series of mini-competitions. In each tournament, we randomly select a few individuals and pick the best one.

    ```python
    import radiate as rd
    
    # Create a tournament selector with 3 competitors per tournament
    selector = rd.TournamentSelector(k=3)
    ```

    **How it works:**
    1. Randomly select `k` individuals from the population
    2. Choose the best individual from this group
    3. Repeat until we have enough individuals

    **When to use:**
    - When you want to balance exploration and exploitation
    - For problems where maintaining diversity is important
    - When you want to avoid premature convergence

    **Parameters:**
    - `k`: Number of competitors in each tournament (default: 3)
        - Smaller k: More random selection
        - Larger k: More selective, favoring better solutions

### 2. Roulette Wheel Selection

??? tip "Best for: Simple problems with well-behaved fitness landscapes"
    Roulette wheel selection (also called fitness-proportional selection) gives each individual a chance of selection proportional to its fitness.

    ```python
    import radiate as rd
    
    selector = rd.RouletteSelector()
    ```

    **How it works:**
    1. Calculate selection probability for each individual:
       $p_i = \frac{f_i}{\sum_{j=1}^N f_j}$
    2. Create a "roulette wheel" where each individual's slice is proportional to its probability
    3. Spin the wheel to select individuals

    **When to use:**
    - When fitness values are positive and well-scaled
    - For problems where you want selection pressure proportional to fitness
    - When you want a simple, effective selection method

    **Watch out for:**
    - Negative fitness values
    - Very large differences in fitness values
    - Premature convergence in some cases

### 3. Boltzmann Selection

??? tip "Best for: Dynamic selection pressure control"
    Boltzmann selection uses a temperature parameter to control selection pressure, similar to simulated annealing.

    ```python
    import radiate as rd
    
    # Create a Boltzmann selector with temperature 4.0
    selector = rd.BoltzmannSelector(temp=4.0)
    ```

    **How it works:**
    1. Adjusts selection probabilities using a temperature parameter
    2. Higher temperature: More random selection
    3. Lower temperature: More selective, favoring better solutions

    **When to use:**
    - When you want to control selection pressure dynamically
    - For problems where you want to start with more exploration and end with more exploitation
    - When you need to balance between exploration and exploitation

    **Parameters:**
    - `temp`: Temperature parameter (default: 1.0)
        - Higher temp: More random selection
        - Lower temp: More selective

### 4. Elite Selection

??? warning "Use with caution: Can lead to premature convergence"
    Elite selection always keeps the best individuals from the current generation.

    ```python
    import radiate as rd
    
    selector = rd.EliteSelector()
    ```

    **How it works:**
    1. Always selects the best `n` individuals
    2. These individuals survive directly to the next generation
    3. Remaining slots filled by other selection methods

    **When to use:**
    - When you want to guarantee preservation of the best solutions
    - For problems where finding the global optimum is critical
    - As a component of a multi-selector strategy

    **Watch out for:**
    - Premature convergence
    - Loss of diversity
    - Local optima trapping

### 5. NSGA-II Selection

??? tip "Best for: Multi-objective optimization problems"
    NSGA-II (Non-dominated Sorting Genetic Algorithm II) is designed for problems with multiple objectives.

    ```python
    import radiate as rd
    
    selector = rd.NSGA2Selector()
    ```

    **How it works:**
    1. Sorts individuals into Pareto fronts based on dominance
    2. Calculates crowding distance for diversity
    3. Selects individuals based on front rank and crowding distance

    **When to use:**
    - For multi-objective optimization problems
    - When you need to maintain a diverse set of solutions
    - When you want to explore trade-offs between objectives

### 6. Stochastic Universal Sampling

??? tip "Best for: Reducing selection bias while maintaining fitness proportionality"
    Stochastic Universal Sampling (SUS) is an improved version of roulette wheel selection that reduces bias.

    ```python
    import radiate as rd
    
    selector = rd.StochasticSamplingSelector()
    ```

    **How it works:**
    1. Creates a single "wheel" with segments proportional to fitness
    2. Uses multiple equally-spaced pointers
    3. Makes a single spin to select all individuals

    **When to use:**
    - When you want to reduce selection bias
    - For problems where maintaining diversity is important
    - When you want more deterministic selection than roulette wheel

### 7. Rank Selection

??? tip "Best for: Problems with poorly scaled fitness values"
    Rank selection bases selection probabilities on the rank of individuals rather than their raw fitness values.

    ```python
    import radiate as rd
    
    selector = rd.RankSelector()
    ```

    **How it works:**
    1. Ranks individuals based on fitness
    2. Assigns selection probabilities based on rank
    3. Selects individuals using these probabilities

    **When to use:**
    - When fitness values are poorly scaled
    - When you want to maintain consistent selection pressure
    - For problems where relative ranking is more important than absolute fitness

## Best Practices

??? tip "Choosing the Right Selector"
    1. **Consider your problem type:**
       - Single objective: Tournament, Roulette, or Boltzmann
       - Multi-objective: NSGA-II
       - Poorly scaled fitness: Rank or Tournament
       - Need for diversity: Tournament or SUS

    2. **Balance exploration and exploitation:**
       - More exploration: Higher tournament size, higher temperature
       - More exploitation: Lower tournament size, lower temperature

    3. **Combine selectors:**
       - Use Elite selection with another selector
       - Consider different selectors for parents and survivors

    4. **Monitor population diversity:**
       - Watch for premature convergence
       - Adjust selection pressure if needed

## Common Pitfalls

??? warning "Things to Watch Out For"
    1. **Premature convergence:**
       - Using too much elite selection
       - Too high selection pressure
       - Solution: Increase tournament size or temperature

    2. **Loss of diversity:**
       - Too much selection pressure
       - Solution: Use tournament selection or increase temperature

    3. **Selection bias:**
       - Poorly scaled fitness values
       - Solution: Use rank selection or normalize fitness values

    4. **Computational cost:**
       - NSGA-II can be expensive for large populations
       - Solution: Consider simpler selectors for large populations