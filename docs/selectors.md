
# Selectors

___
Selectors in a genetic algorith are responsible for selecting individuals from the population to create the
next generation. Radiate does this in two ways, selecting for survivors and selecting for offspring. The
survivors get passed directly down into the new generation, while the offspring are used to create new
individuals through genetic operators like mutation and crossover. The `Population<C>` is treated as a 
distribution of individuals, and the selector is responsible for sampling from this distribution to create
the next generation.

The selection process is a critical part of the genetic algorithm, as it determines which individuals will
be passed on to the next generation and which will be discarded. A majority of the time t
he selection process is based on the fitness of the individuals in the population, with fitter individuals being more likely to be selected. The choice of selection strategy can have a significant impact on the performance of the genetic algorithm, so it is important to choose a selection strategy that is well-suited to the problem being solved.

Radiate defines a selector as:

* `population` - The population of individuals to select from.
* `objective` - The optimization goal of the genetic algorithm.
* `count` - The number of individuals to select.
```rust
pub trait Select<C: Chromosome> {
    fn name(&self) -> &'static str;

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C>;
}
```
!!! note

    1. The incoming `Population<C>` will always be sorted in the order of the `Objective` provided.
    2. The `Population<C>` is immutable, meaning the selector must return a new `Population<C>` with `Clone`d individuals.

Radiate provides a number of built-in selectors that can be used to customize the selection process, or you can define your own custom selectors
___

## Elite

The `EliteSelector` is a selection strategy that selects the top `n` individuals from the population based on their fitness values. This can be useful for preserving the best individuals in the population and preventing them from being lost during the selection process. 

!!! warning 

    This is a deterministic selection strategy that always selects the same individuals from the population. This can lead to premature convergence and lack of diversity in the population, so it is important to use it judiciously.

Create a new `EliteSelector`
```rust
let selector = EliteSelector::new();
```

## Tournament

> Inputs
> 
>   * `num`: usize - The number of individuals to compete in each tournament.

The `TournamentSelector` is a selection strategy that selects individuals from the population by holding a series of tournaments. In each tournament, a random subset of individuals is selected, and the fittest individual from that subset is chosen. This can help to maintain diversity in the population and prevent premature convergence by allowing weaker individuals to be selected occasionally.

Create a new `TournamentSelector` with a tournament size of 3
```rust
let selector = TournamentSelector::new(3);
```

## Roulette

The `RouletteSelector` is a selection strategy that selects individuals from the population based on their fitness values. The probability of an individual being selected is proportional to its fitness value, so fitter individuals are more likely to be chosen. The probability of an individual being selected is

$$
p_{i}={\frac {f_{i}}{\Sigma _{j=1}^{N}f_{j}}}
$$

where $p_{i}$ is the probability of individual $i$ being selected, $f_{i}$ is the fitness value of individual $i$, and $N$ is the total number of individuals in the population.
 This is an extremely popular selection strategy due to its simplicity and effectiveness. Due to 
the random nature of the selection process, it can help to maintain diversity in the population and prevent premature convergence.

Create a new `RouletteSelector`
```rust
let selector = RouletteSelector::new();
```

## Boltzmann

> Inputs
> 
>   * `temperature`: f32 - The temperature of the selection process.

The `BoltzmannSelector` is a probabilistic selection strategy inspired by the Boltzmann distribution from statistical mechanics, where selection probabilities are scaled based on temperature. Temperature influences the balance between exploration and exploitation during the algorithm’s run.

As the temperature decreases, the selection process becomes more deterministic, with fitter individuals being more likely to be selected. Conversely, as the temperature increases, the selection process becomes more random, with all individuals having an equal chance of being selected.

Create a new `BoltzmannSelector` with a temperature of 0.1
```rust
let selector = BoltzmannSelector::new(0.1);
```

## NSGA-II

The `NSGA2Selector` is a selection strategy used in multi-objective optimization problems. It is based on the Non-Dominated Sorting Genetic Algorithm II (NSGA-II) and selects individuals based on their Pareto dominance rank and crowding distance. The NSGA-II algorithm is designed to maintain a diverse set of solutions that represent the trade-offs between multiple conflicting objectives.

* Individuals are first sorted into Pareto fronts based on their dominance relationships.
* Individuals in the same front are then ranked based on their crowding distance, which measures the density of solutions around them.
* Individuals with lower ranks and higher crowding distances are more likely to be selected.

Create a new `NSGA2Selector`
```rust
let selector = NSGA2Selector::new();
```

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
  
Create a new `StochasticUniversalSamplingSelector`
```rust
let selector = StochasticUniversalSamplingSelector::new();
```