
# Selectors

___
Selectors in a genetic algorithm are responsible for selecting individuals from the population to create the
next generation. Radiate does this in two ways, selecting for survivors and selecting for offspring. The
survivors get passed directly down into the new generation, while the offspring are used to create new
individuals through `alters` like `mutation` and `crossover`. The `population` is treated as a 
distribution of individuals, and the `selector` is responsible for sampling from this distribution to create
the next generation.

The selection process is a critical part of the genetic algorithm, as it determines which individuals will
be passed on to the next generation and which will be discarded. A majority of the time the selection process is based on the fitness of the individuals in the population, with fitter individuals being more likely to be selected. The choice of selection strategy can have a significant impact on the performance of the genetic algorithm, so it is important to choose a selection strategy that is well-suited to the problem being solved.

Radiate provides a number of built-in selectors that can be used to customize the selection process.

## Overview

| Selector | Best For | Selection Pressure | Diversity |
|----------|----------|-------------------|-----------|
| [Tournament](#tournament) | Most problems | Adjustable (via k) | High |
| [Roulette](#roulette) | Most problems | High | Medium |
| [Boltzmann](#boltzmann) | Most problems | Adjustable (via temp) | Medium-High |
| [Elite](#elite) | Preserving best solutions | Very High | Low |
| [NSGA-II](#nsga-ii) | Multi-objective problems | Balanced | High |
| [Stochastic](#stochastic-universal-sampling) | Reducing bias | Medium | High |
| [Rank](#rank) | Erratic fitness | Medium | High |
| [Linear Rank](#linear-rank) | Linear scaling | Adjustable (via pressure) | Medium-High |
| [Random](#random) | Testing | Low | High |
| [Steady State](#steady-state) | Maintaining population stability | Low | Medium |
| [Tournament NSGA-II](#tournament-nsga-ii) | Multi-objective problems | Balanced | High |

___

## Elite

The `EliteSelector` is a selection strategy that selects the top `n` individuals from the population based on their fitness values. This can be useful for preserving the best individuals in the population and preventing them from being lost during the selection process. 

!!! warning 

    This is a deterministic selection strategy that always selects the same individuals from the population. This can lead to premature convergence and lack of diversity in the population, so it is important to use it judiciously.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.EliteSelector()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = EliteSelector::new();
    ```

---

## Tournament

> Inputs
> 
>   * `num`: usize - The number of individuals to compete in each tournament.

The `TournamentSelector` is a selection strategy that selects individuals from the `population` by holding a series of tournaments. In each tournament, a random subset of size `num` of individuals is selected, and the fittest individual from that subset is chosen. This can help to maintain diversity in the `population` and prevent premature convergence by allowing weaker individuals to be selected occasionally.

Create a new `TournamentSelector` with a tournament size of 3

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.TournamentSelector(k=3)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = TournamentSelector::new(3);
    ```

---

## Roulette

The `RouletteSelector` is a selection strategy that selects individuals from the `population` based on their fitness values. The probability of an individual being selected is proportional to its fitness value, so fitter individuals are more likely to be chosen. The probability of an individual being selected is:

$$
p_{i}={\frac {f_{i}}{\Sigma _{j=1}^{N}f_{j}}}
$$

Where

- $p_{i}$ is the probability of individual $i$ being selected
- $f_{i}$ is the fitness value of individual $i$
- $N$ is the total number of individuals in the `population`.
 
This is an extremely popular selection strategy due to its simplicity and effectiveness. Due to the random nature of the selection process, it can help to maintain diversity in the `population` and prevent premature convergence.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.RouletteSelector()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = RouletteSelector::new();
    ```

---

## Boltzmann

> Inputs
> 
>   * `temperature`: f32 - The temperature of the selection process.

The `BoltzmannSelector` is a probabilistic selection strategy inspired by the Boltzmann distribution from statistical mechanics, where selection probabilities are scaled based on temperature. Temperature influences the balance between exploration and exploitation during the algorithm’s run.

As the temperature decreases, the selection process becomes more deterministic, with fitter individuals being more likely to be selected. Conversely, as the temperature increases, the selection process becomes more random, with all individuals having an equal chance of being selected.

=== ":fontawesome-brands-python: Python"

    If the `temperature` is not specified, it defaults to 1.0.

    ```python
    import radiate as rd

    selector = rd.BoltzmannSelector(temp=4.0)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = BoltzmannSelector::new(4_f32);
    ```
---

## NSGA-II

The `NSGA2Selector` is a selection strategy used in multi-objective optimization problems. It is based on the Non-Dominated Sorting Genetic Algorithm II (NSGA-II) and selects individuals based on their Pareto dominance rank and crowding distance. The NSGA-II algorithm is designed to maintain a diverse set of solutions that represent the trade-offs between multiple conflicting objectives.

* Individuals are first sorted into Pareto fronts based on their dominance relationships.
* Individuals in the same front are then ranked based on their crowding distance, which measures the density of solutions around them.
* Individuals with lower ranks and higher crowding distances are more likely to be selected.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.NSGA2Selector()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = NSGA2Selector::new();
    ```
---

## Tournament NSGA-II

The `TournamentNSGA2Selector` is a selection strategy that combines the principles of tournament selection with the NSGA-II algorithm. It selects individuals based on their Pareto dominance rank and crowding distance, but uses a tournament-style approach to select individuals from each Pareto front.

* Individuals are first sorted into Pareto fronts based on their dominance relationships.
* A tournament is held within each Pareto front, where a random subset of individuals is selected.
* The winner of the tournament is selected based on their crowding distance, which measures the density of solutions around them.
  
=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.TournamentNSGA2Selector()
    ```
=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = TournamentNSGA2Selector::new();
    ```

---

## Stochastic Universal Sampling

Stochastic Universal Sampling (SUS) is a probabilistic selection technique used to ensure that selection is proportional to fitness, while maintaining diversity. Some consider it an improvement over roulette wheel selection, designed to reduce bias and randomness in the selection process by ensuring all individuals have a chance to be chosen, proportional to their fitness values.

1. Fitness Proportional Selection:
  	* Each individual in the population is assigned a segment of a virtual “roulette wheel,” where the size of the segment is proportional to the individual’s fitness.
	* Individuals with higher fitness occupy larger segments.
* Single Spin with Multiple Pointers:
    * Unlike traditional roulette wheel selection, which spins the wheel multiple times (once per selection), SUS uses a single spin and places multiple evenly spaced pointers on the wheel.
	* The distance between the pointers is: `d = total_fitness / n`, where `n` is the number of individuals to select.
* Selection:
    * The wheel is spun once, and the pointers are placed on the wheel at random positions.
    * Individuals whose segments are intersected by the pointers are selected.
  
=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.StochasticSamplingSelector()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = StochasticUniversalSamplingSelector::new();
    ```

---

## Rank

The `RankSelector` is a selection strategy that selects individuals from the `population` based on their rank, or index, in the `population`. The fitness values of the individuals are first ranked, and then the selection probabilities are assigned based on these ranks. This helps to maintain diversity in the population and prevent premature convergence by ensuring that all individuals have a chance to be selected, regardless of their fitness values. The selection probabilities are calculated as follows:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.RandkSelector()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = RankSelector::new();
    ```

---

## Linear Rank

> Inputs
>
> * `pressure`: f32 - The scaling factor for the selection probabilities.

The `LinearRankSelector` is a selection strategy that selects individuals from the `population` based on their rank, or index, in the `population`, but with a linear scaling of the selection probabilities. The fitness values of the individuals are first ranked, and then the scaling factor is applied to the ranks. This helps to maintain diversity in the `population` and prevent premature convergence by ensuring that all individuals have a chance to be selected, but with a bias towards fitter individuals. The linear scaling function is defined as follows:

$$
p_{i}={\Sigma _{i=1}^{N}r_{i}} * pressure
$$

Where

- $p_{i}$ is the probability of individual $i$ being selected
- $r_{i}$ is the rank of individual $i$
- $N$ is the total number of individuals in the `population` 
- `pressure` is a scaling factor that can be adjusted to control

A higher `pressure` will result in a stronger bias towards fitter individuals, while a lower value will result in a more uniform selection.

=== ":fontawesome-brands-python: Python"

    If `pressure` is not specified, it defaults to `0.5`.

    ```python
    import radiate as rd

    selector = rd.LinearRankSelector(pressure=0.1)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = LinearRankSelector::new(0.1);
    ```

---

## Random

The `RandomSelector` is a selection strategy that selects individuals from the `population` at random. It allows all individuals to have an equal chance of being selected, regardless of their fitness values. There is no bias towards fitter individuals, making it a simple and straightforward selection strategy. However, it may not be as effective in maintaining diversity in the `population` and preventing premature convergence as other selection strategies. Keep in mind, the goal of a genetic algorithm is to evolve a `population` of individuals towards a specific target over time, and random selection does not take advantage of the information provided by the fitness values of the individuals. This selection strategy is mainly used for testing purposes in Radiate, but may be useful in some specific scenarios.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.RandomSelector()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = RandomSelector::new();
    ```

---

## Steady State

> Inputs
>
> * `replacement_count`: usize - The number of individuals to replace in the population.

The `SteadyStateSelector` is a selection strategy that selects individuals from the `population` based on their fitness values, but with a focus on maintaining a steady state in the `population`. This means that the selection process is designed to prevent drastic changes in the `population` from one generation to the next, and to ensure that the best individuals are preserved while still allowing for some degree of exploration and diversity. We do this by coping the original `population`, then taking `replacement_count` random individuals from the current `population` and inserting them at a random index into the resulting `population`. This helps to maintain a balance between exploration and exploitation in the selection process.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    selector = rd.SteadyStateSelector(replacements=10)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let selector = SteadyStateSelector::new(10);
    ```

---

## Example

Let's continue with our example from the previous section - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll use the same `codec` and `fitness_function` as before, but now we'll incorporate a `selector` to choose individuals for the next generation.

=== ":fontawesome-brands-python: Python"

    ```python
    from typing import List
    import radiate as rd

    # Define a fitness function that uses the decoded values
    def fitness_function(individual: List[float]) -> float:    
        # Calculate how well these parameters fit your data
        a = individual[0]
        b = individual[1]
        return calculate_error(a, b)  # Your error calculation here

    # Create a codec for two parameters (a and b)
    codec = rd.FloatCodec.vector(
        length=2,                   # We need two parameters: a and b
        value_range=(-1.0, 1.0),    # Start with values between -1 and 1
        bound_range=(-10.0, 10.0)   # Allow evolution to modify the values between -10 and 10
    )

    # Use Boltzmann selection for offspring - individuals which
    # will be used to create new individuals through mutation and crossover
    offspring_selector = rd.BoltzmannSelector(temp=4)

    # Use tournament selection for survivors - individuals which will 
    # be passed down unchanged to the next generation
    survivor_selector = rd.TournamentSelector(k=3)

    # Create the evolution engine
    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_function,
        offspring_selector=offspring_selector,
        survivor_selector=survivor_selector,
        # ... other parameters ...
    )

    # Run the engine
    result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(1000)])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Define a fitness function that uses the decoded values
    fn fitness_fn(individual: Vec<f32>) -> f32 {
        let a = individual[0];
        let b = individual[1];
        calculate_error(a, b)  // Your error calculation here
    }

    // This will produce a Genotype<FloatChromosome> with 1 FloatChromosome which
    // holds 2 FloatGenes (a and b), each with a value between -1.0 and 1.0 and a bound between -10.0 and 10.0
    let codec = FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0);

    // Use Boltzmann selection for offspring - individuals which
    // will be used to create new individuals through mutation and crossover
    let offspring_selector = BoltzmannSelector::new(4.0);

    // Use tournament selection for survivors - individuals which will
    // be passed down unchanged to the next generation
    let survivor_selector = TournamentSelector::new(3);

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(offspring_selector)
        .survivor_selector(survivor_selector)
        .fitness_fn(fitness_fn)
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ```