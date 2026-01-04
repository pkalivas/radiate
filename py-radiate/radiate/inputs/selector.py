from typing import Any

from radiate.genome.population import Population
from radiate.inputs.input import EngineInput, EngineInputType
from .component import ComponentBase
from ..genome import GENE_TYPE_MAPPING, GeneType


class SelectorBase(ComponentBase):
    def __init__(
        self,
        component: str,
        args: dict[str, Any] = {},
        allowed_genes: set[GeneType] | GeneType = {},
    ):
        super().__init__(component=component, args=args)
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.all()

    def __str__(self):
        """
        Return a string representation of the selector.
        :return: String representation of the selector.
        """
        return f"Selector(name={self.component}, args={self.args})"

    def __repr__(self):
        """
        Return a detailed string representation of the selector.
        :return: Detailed string representation of the selector.
        """
        return f"SelectorBase(selector={self.component}, args={self.args}, allowed_genes={self.allowed_genes})"

    def __eq__(self, value):
        if not isinstance(value, SelectorBase):
            return False
        return (
            self.component == value.component
            and self.args == value.args
            and self.allowed_genes == value.allowed_genes
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

        selector_input = EngineInput(
            component=self.component,
            input_type=EngineInputType.SurvivorSelector,
            args=self.args,
        ).__backend__()

        objective_input = EngineInput(
            component="Objective",
            input_type=EngineInputType.Objective,
            allowed_genes={gene_type},
            args={"objective": "|".join(objective)}
            if isinstance(objective, list)
            else {"objective": objective},
        ).__backend__()

        return Population.from_rust(
            py_select(
                gene_type=GENE_TYPE_MAPPING["rs"][gene_type],
                selector=selector_input,
                objective=objective_input,
                population=population.__backend__(),
                count=count,
            )
        )


class TournamentSelector(SelectorBase):
    def __init__(self, k: int = 3):
        """
        The `TournamentSelector` is a selection strategy that selects individuals from the `population` by
        holding a series of tournaments. In each tournament, a random subset of size `k` of individuals
        is selected, and the fittest individual from that subset is chosen. This can help to maintain
        diversity in the `population` and prevent premature convergence by allowing weaker individuals to be selected occasionally.

        :param k: Tournament size.
        """
        super().__init__(component="TournamentSelector", args={"k": k})


class RouletteSelector(SelectorBase):
    def __init__(self):
        """
        The `RouletteSelector` is a selection strategy that selects individuals from the `population`
        based on their fitness values. The probability of an individual being selected is proportional
        to its fitness value, so fitter individuals are more likely to be chosen. The probability of an
        individual being selected can be thought of as:

        P(i) = f(i) / Σ f(j) for all j in population

        Although the implementation itself is a bit more mathematically complex to ensure accuracy.
        This is an extremely popular selection strategy due to its simplicity and effectiveness.
        """
        super().__init__(component="RouletteSelector")


class RankSelector(SelectorBase):
    def __init__(self):
        """
        The `RankSelector` is a selection strategy that selects individuals from the `population`
        based on their rank, or index, in the `population`. The fitness values of the individuals are
        first ranked, and then the selection probabilities are assigned based on these ranks.
        This helps to maintain diversity in the population and prevent premature convergence by ensuring that
        all individuals have a chance to be selected, regardless of their fitness values.
        """
        super().__init__(component="RankSelector")


class EliteSelector(SelectorBase):
    def __init__(self):
        """
        The `EliteSelector` is a selection strategy that selects the top `n` individuals from the population
        based on their fitness values. This can be useful for preserving the best individuals
        in the population and preventing them from being lost during the selection process.
        """
        super().__init__(component="EliteSelector")


class BoltzmannSelector(SelectorBase):
    def __init__(self, temp: float = 1.0):
        """
        The `BoltzmannSelector` is a probabilistic selection strategy inspired by the Boltzmann distribution
        from statistical mechanics, where selection probabilities are scaled based on temperature.
        Temperature influences the balance between exploration and exploitation during the algorithm’s run.

        As the temperature decreases, the selection process becomes more deterministic,
        with fitter individuals being more likely to be selected. Conversely, as the temperature increases,
        the selection process becomes more random, with all individuals having an equal chance of being selected.

        :param temp: Temperature for the Boltzmann selector.
        """
        super().__init__(component="BoltzmannSelector", args={"temp": temp})


class StochasticSamplingSelector(SelectorBase):
    def __init__(self):
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
        super().__init__(component="StochasticUniversalSamplingSelector")


class LinearRankSelector(SelectorBase):
    def __init__(self, pressure: float = 0.5):
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
        super().__init__(component="LinearRankSelector", args={"pressure": pressure})


class NSGA2Selector(SelectorBase):
    def __init__(self):
        """
        The `NSGA2Selector` is a selection strategy used in multi-objective optimization problems.
         
        It is based on the Non-Dominated Sorting Genetic Algorithm II (NSGA-II) and selects
        individuals based on their Pareto dominance rank and crowding distance. The NSGA-II algorithm 
        is designed to maintain a diverse set of solutions that represent the trade-offs between multiple conflicting objectives.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * Individuals in the same front are then ranked based on their crowding distance, which measures the density of solutions around them.
        * Individuals with lower ranks and higher crowding distances are more likely to be selected.
        """
        super().__init__(component="NSGA2Selector")


class TournamentNSGA2Selector(SelectorBase):
    def __init__(self, k: int = 3):
        """
        The `TournamentNSGA2Selector` is a selection strategy that combines the principles of tournament
        selection with the NSGA-II algorithm. It selects individuals based on their Pareto dominance 
        rank and crowding distance, but uses a tournament-style approach to select individuals from each Pareto front.

        * Individuals are first sorted into Pareto fronts based on their dominance relationships.
        * A tournament is held within each Pareto front, where a random subset of individuals is selected.
        * The winner of the tournament is selected based on their crowding distance, which measures the density of solutions around them.
    
        :param k: Tournament size.
        """
        super().__init__(component="TournamentNSGA2Selector", args={"k": k})


class SteadyStateSelector(SelectorBase):
    def __init__(self, replacement_count: int = 10):
        """
        The `SteadyStateSelector` is a selection strategy that selects individuals
        from the `population` based on their fitness values, but with a focus on maintaining a steady state
        in the `population`. This means that the selection process is designed to prevent drastic 
        changes in the `population` from one generation to the next, and to ensure that the best individuals 
        are preserved while still allowing for some degree of exploration and diversity. We do this by 
        copying the original `population`, then taking `replacement_count` random individuals from 
        the current `population` and inserting them at a random index into the resulting `population`. 
        """
        super().__init__(
            component="SteadyStateSelector",
            args={"replacement_count": replacement_count},
        )
