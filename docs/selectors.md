
# Selectors

___
Selectors in a genetic algorithm are responsible for selecting individuals from the population to create the
next generation. Radiate does this in two ways, selecting for survivors and selecting for offspring. The
survivors get passed directly down into the new generation, while the offspring are used to create new
individuals through genetic operators like mutation and crossover. The `Population<C>` is treated as a 
distribution of individuals, and the selector is responsible for sampling from this distribution to create
the next generation.

The selection process is a critical part of the genetic algorithm, as it determines which individuals will
be passed on to the next generation and which will be discarded. A majority of the time the selection process is based on the fitness of the individuals in the population, with fitter individuals being more likely to be selected. The choice of selection strategy can have a significant impact on the performance of the genetic algorithm, so it is important to choose a selection strategy that is well-suited to the problem being solved.

Radiate defines a selector as:

* `population` - The population of individuals to select from.
* `objective` - The optimization goal of the genetic algorithm.
* `count` - The number of individuals to select.
```rust
pub trait Select<C: Chromosome> {
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

The `TournamentSelector` is a selection strategy that selects individuals from the population by holding a series of tournaments. In each tournament, a random subset of size `num` of individuals is selected, and the fittest individual from that subset is chosen. This can help to maintain diversity in the population and prevent premature convergence by allowing weaker individuals to be selected occasionally.

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

## Rank

The `RankSelector` is a selection strategy that selects individuals from the population based on their rank in the population. The fitness values of the individuals are first ranked, and then the selection probabilities are assigned based on these ranks. This helps to maintain diversity in the population and prevent premature convergence by ensuring that all individuals have a chance to be selected, regardless of their fitness values. The selection probabilities are calculated as follows:

$$
p_{i}={\frac {r_{i}}{\Sigma _{j=1}^{N}r_{j}}}
$$

where $p_{i}$ is the probability of individual $i$ being selected, $r_{i}$ is the rank of individual $i$, and $N$ is the total number of individuals in the population. The rank of an individual is determined by its position in the sorted population, with the best individual having a rank of 1, the second best having a rank of 2, and so on.

Create a new `RankSelector`
```rust
let selector = RankSelector::new();
```

## Linear Rank

> Inputs
>
> * `sp`: f32 - The scaling factor for the selection probabilities.

The `LinearRankSelector` is a selection strategy that selects individuals from the population based on their rank in the population, but with a linear scaling of the selection probabilities. The fitness values of the individuals are first ranked, and then the selection probabilities are assigned based on these ranks using a linear function. This helps to maintain diversity in the population and prevent premature convergence by ensuring that all individuals have a chance to be selected, but with a bias towards fitter individuals. The linear scaling function is defined as follows:

$$
p_{i}={\Sigma _{i=1}^{N}r_{i}} * sp
$$

where $p_{i}$ is the probability of individual $i$ being selected, $r_{i}$ is the rank of individual $i$, and $N$ is the total number of individuals in the population. The `sp` parameter is a scaling factor that can be adjusted to control the selection pressure. A higher value of `sp` will result in a stronger bias towards fitter individuals, while a lower value will result in a more uniform selection.


Create a new `LinearRankSelector`
```rust
let selector = LinearRankSelector::new(0.1);
```

## Random

The `RandomSelector` is a selection strategy that selects individuals from the population at random. It allows all individuals to have an equal chance of being selected, regardless of their fitness values. There is no bias towards fitter individuals, making it a simple and straightforward selection strategy. However, it may not be as effective in maintaining diversity in the population and preventing premature convergence as other selection strategies. Keep in mind, the goal of a genetic algorithm is to evolve a population of individuals towards a specific target over time, and random selection does not take advantage of the information provided by the fitness values of the individuals. This selection strategy is mainly used for testing purposes in Radiate, but may be useful in some specific scenarios.

Create a new `RandomSelector`
```rust
let selector = RandomSelector::new();
```

## Steady State

> Inputs
>
> * `num`: usize - The number of individuals to replace in the population.

The `SteadyStateSelector` is a selection strategy that selects individuals from the population based on their fitness values, but with a focus on maintaining a steady state in the population. This means that the selection process is designed to prevent drastic changes in the population from one generation to the next, and to ensure that the best individuals are preserved while still allowing for some degree of exploration and diversity. We do this by creating a new population with the best individuals, then taking `num` random individuals from the current population and inserting them at a random index into the resulting population. This helps to maintain a balance between exploration and exploitation in the selection process.

Create a new `SteadyStateSelector` with a replacement size of 10
```rust
let selector = SteadyStateSelector::new(10);
```