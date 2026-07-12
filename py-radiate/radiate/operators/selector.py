from __future__ import annotations

from .._rd import components
from ..genome import GENE_TYPE_MAPPING
from ..genome.population import Population
from .input import EngineInput, EngineInputType


class Select(EngineInput):
    def __init__(self, component: str, **kwargs):
        super().__init__(
            component=component,
            input_type=EngineInputType.Unknown,
            **kwargs,
        )

    def select(
        self, population: Population, objective: list[str] | str, count: int
    ) -> Population:
        """
        Select individuals from the population based on the selector's criteria.
        :param population: The population to select from.
        :param objective: The objective function or criteria for selection.
        :param count: The number of individuals to select.
        :return: A new population containing the selected individuals.
        """
        from radiate.radiate import py_select

        gene_type = population.gene_type()

        objectives = objective if isinstance(objective, list) else [objective]

        objective_input = EngineInput(
            input_type=EngineInputType.Objective,
            objective=objectives,
        ).__backend__()

        return Population.from_rust(
            py_select(
                gene_type=GENE_TYPE_MAPPING["rs"][gene_type],
                selector=self.to_offspring_selector().__backend__(),
                objective=objective_input,
                population=population.__backend__(),
                count=count,
            )
        )

    def to_survivor_selector(self) -> Select:
        self._input_type = EngineInputType.SurvivorSelector
        return self

    def to_offspring_selector(self) -> Select:
        self._input_type = EngineInputType.OffspringSelector
        return self

    @staticmethod
    def tournament(k: int = 3) -> Select:
        """
        The `TournamentSelector` is a selection strategy that selects individuals from the `population` by
        holding a series of tournaments. In each tournament, a random subset of size `k` of individuals
        is selected, and the fittest individual from that subset is chosen. This can help to maintain
        diversity in the `population` and prevent premature convergence by allowing weaker individuals to be selected occasionally.

        :param k: Tournament size.
        """
        return Select(components.TOURNAMENT_SELECTOR, k=k)

    @staticmethod
    def roulette() -> Select:
        """
        The `RouletteSelector` is a selection strategy that selects individuals from the `population`
        based on their fitness values. The probability of an individual being selected is proportional
        to its fitness value, so fitter individuals are more likely to be chosen. The probability of an
        individual being selected can be thought of as:

        P(i) = f(i) / Σ f(j) for all j in population

        Although the implementation itself is a bit more mathematically complex to ensure accuracy.
        This is an extremely popular selection strategy due to its simplicity and effectiveness.
        """
        return Select(components.ROULETTE_WHEEL_SELECTOR)

    @staticmethod
    def nsga2() -> Select:
        """
        The `NSGA2Selector` is a selection strategy used in multi-objective optimization problems.

        It is based on the Non-Dominated Sorting Genetic Algorithm II (NSGA-II) and selects
        individuals based on their Pareto dominance rank and crowding distance. The NSGA-II algorithm
        is designed to maintain a diverse set of solutions that represent the trade-offs between multiple conflicting objectives.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * Individuals in the same front are then ranked based on their crowding distance, which measures the density of solutions around them.
        * Individuals with lower ranks and higher crowding distances are more likely to be selected.
        """
        return Select(components.NSGA2_SELECTOR)

    @staticmethod
    def nsga3(points: int = 12) -> Select:
        """
        The `NSGA3Selector` is a selection strategy used in multi-objective optimization problems, based on the NSGA-III algorithm. It extends the NSGA-II algorithm by introducing reference points to guide the selection process towards a well-distributed set of solutions across the Pareto front.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * Reference points are generated in the objective space to represent desired trade-offs between objectives.
        * Individuals are selected based on their proximity to these reference points, ensuring a diverse set of solutions across the Pareto front.

        :param points: The number of reference points to use for guiding the selection process.
        """
        return Select(components.NSGA3_SELECTOR, points=points)

    @staticmethod
    def elite() -> Select:
        """
        The `EliteSelector` is a selection strategy that selects the top `n` individuals from the population
        based on their fitness values. This can be useful for preserving the best individuals
        in the population and preventing them from being lost during the selection process.
        """
        return Select(components.ELITE_SELECTOR)

    @staticmethod
    def boltzmann(temp: float = 1.0) -> Select:
        """
        The `BoltzmannSelector` is a probabilistic selection strategy inspired by the Boltzmann distribution
        from statistical mechanics, where selection probabilities are scaled based on temperature.
        Temperature influences the balance between exploration and exploitation during the algorithm’s run.

        As the temperature decreases, the selection process becomes more deterministic,
        with fitter individuals being more likely to be selected. Conversely, as the temperature increases,
        the selection process becomes more random, with all individuals having an equal chance of being selected.

        :param temp: Temperature for the Boltzmann selector.
        """
        return Select(components.BOLTZMANN_SELECTOR, temp=temp)

    @staticmethod
    def rank() -> Select:
        """
        The `RankSelector` is a selection strategy that selects individuals from the `population`
        based on their rank, or index, in the `population`. The fitness values of the individuals are
        first ranked, and then the selection probabilities are assigned based on these ranks.
        This helps to maintain diversity in the population and prevent premature convergence by ensuring that
        all individuals have a chance to be selected, regardless of their fitness values.
        """
        return Select(components.RANK_SELECTOR)

    @staticmethod
    def linear_rank(pressure: float = 1.5) -> Select:
        """
        The `LinearRankSelector` is a selection strategy that selects individuals from the
        `population` based on their rank, or index, in the `population`, but with a linear scaling
        of the selection probabilities. The fitness values of the individuals are first ranked, and
        then the scaling factor is applied to the ranks. This helps to maintain diversity
        in the `population` and prevent premature convergence by ensuring that all individuals have a
        chance to be selected, but with a bias towards fitter individuals. The linear scaling function can be thought of as:

        P(i) = (2 - pressure) / N + (2 * (rank(i) - 1) * (pressure - 1)) / (N * (N - 1))

        A higher `pressure` will result in a stronger bias towards fitter individuals, while a lower value will result in a more uniform selection.

        :param pressure: Pressure for the linear rank selector.
        """
        return Select(components.LINEAR_RANK_SELECTOR, pressure=pressure)

    @staticmethod
    def stochastic_universal_sampling() -> Select:
        """
        Stochastic Universal Sampling (SUS) is a probabilistic selection technique used to ensure that selection is
        proportional to fitness, while maintaining diversity. Some consider it an improvement over roulette wheel selection,
        designed to reduce bias and randomness in the selection process by ensuring all individuals have a chance to be chosen,
        proportional to their fitness values.

        1. Fitness Proportional Selection:
            * Each individual in the population is assigned a segment of a virtual “roulette wheel,” where the size of
              the segment is proportional to the individual's fitness.
            * Individuals with higher fitness occupy larger segments.
        * Single Spin with Multiple Pointers:
            * Unlike traditional roulette wheel selection, which spins the wheel multiple times (once per selection), SUS
              uses a single spin and places multiple evenly spaced pointers on the wheel.
            * The distance between the pointers is: `d = total_fitness / n`, where `n` is the number of individuals to select.
        * Selection:
            * The wheel is spun once, and the pointers are placed on the wheel at random positions.
            * Individuals whose segments are intersected by the pointers are selected.
        """
        return Select(components.STOCHASTIC_UNIVERSAL_SELECTOR)

    @staticmethod
    def tournament_nsga2(k: int = 3) -> Select:
        """
        The `TournamentNSGA2Selector` is a selection strategy that combines the principles of tournament
        selection with the NSGA-II algorithm. It selects individuals based on their Pareto dominance
        rank and crowding distance, but uses a tournament-style approach to select individuals from each Pareto front.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * A tournament is held within each Pareto front, where a random subset of individuals is selected.
        * The winner of the tournament is selected based on their crowding distance, which measures the density of solutions around them.

        :param k: Tournament size.
        """
        return Select(components.TOURNAMENT_NSGA2_SELECTOR, k=k)
