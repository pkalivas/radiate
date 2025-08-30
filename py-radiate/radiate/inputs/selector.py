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
        Initialize the tournament selector with tournament size.
        :param k: Tournament size.
        """
        super().__init__(component="TournamentSelector", args={"k": k})


class RouletteSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the roulette selector.
        """
        super().__init__(component="RouletteSelector")


class RankSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the rank selector.
        """
        super().__init__(component="RankSelector")


class EliteSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the elite selector.
        """
        super().__init__(component="EliteSelector")


class BoltzmannSelector(SelectorBase):
    def __init__(self, temp: float = 1.0):
        """
        Initialize the Boltzmann selector with temperature.
        :param temp: Temperature for the Boltzmann selector.
        """
        super().__init__(component="BoltzmannSelector", args={"temp": temp})


class StochasticSamplingSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the stochastic sampling selector.
        """
        super().__init__(component="StochasticUniversalSamplingSelector")


class LinearRankSelector(SelectorBase):
    def __init__(self, pressure: float = 0.5):
        """
        Initialize the linear rank selector.
        :param pressure: Pressure for the linear rank selector.
        """
        super().__init__(component="LinearRankSelector", args={"pressure": pressure})


class NSGA2Selector(SelectorBase):
    def __init__(self):
        """
        Initialize the NSGA2 selector.
        """
        super().__init__(component="NSGA2Selector")


class TournamentNSGA2Selector(SelectorBase):
    def __init__(self, k: int = 3):
        """
        Initialize the Tournament NSGA2 selector with tournament size.
        :param k: Tournament size.
        """
        super().__init__(component="TournamentNSGA2Selector", args={"k": k})


class SteadyStateSelector(SelectorBase):
    def __init__(self, replacement_count: int = 10):
        """
        Initialize the steady state selector.
        """
        super().__init__(
            component="SteadyStateSelector",
            args={"replacement_count": replacement_count},
        )
