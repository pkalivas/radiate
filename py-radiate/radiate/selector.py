from typing import Dict, Any
from .param import EngineParam

from radiate.radiate import Selector

class SelectorBase:
    def __init__(self, selector: Selector):
        """
        Initialize the selector base with a Selector instance.
        :param selector: An instance of Selector.
        """
        self.selector = selector

    def __str__(self):
        """
        Return a string representation of the selector.
        :return: String representation of the selector.
        """
        return f"Selector(name={self.selector.name}, args={self.selector.args})"
    
    def __repr__(self):
        """
        Return a detailed string representation of the selector.
        :return: Detailed string representation of the selector.
        """
        return f"SelectorBase(selector={self.selector})"
    

class TournamentSelector(SelectorBase):
    def __init__(self, k: int = 3):
        """
        Initialize the tournament selector with tournament size.
        :param k: Tournament size.
        """
        selector = Selector.tournament_selector(tournament_size=k)
        super().__init__(selector)

class RouletteSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the roulette selector.
        """
        selector = Selector.roulette_wheel_selector()
        super().__init__(selector)

class RankSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the rank selector.
        """
        selector = Selector.rank_selector()
        super().__init__(selector)

class EliteSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the elite selector.
        """
        selector = Selector.elite_selector()
        super().__init__(selector)

class BoltzmannSelector(SelectorBase):
    def __init__(self, temp: float = 1.0):
        """
        Initialize the Boltzmann selector with temperature.
        :param temp: Temperature for the Boltzmann selector.
        """
        selector = Selector.boltzmann_selector(temp=temp)
        super().__init__(selector)

class StochasticSamplingSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the stochastic sampling selector.
        """
        selector = Selector.stochastic_sampling_selector()
        super().__init__(selector)

class LinearRankSelector(SelectorBase):
    def __init__(self, pressure: float = 0.5):
        """
        Initialize the linear rank selector.
        :param pressure: Pressure for the linear rank selector.
        """
        selector = Selector.linear_rank_selector(pressure=pressure)
        super().__init__(selector)

class NSGA2Selector(SelectorBase):
    def __init__(self):
        """
        Initialize the NSGA2 selector.
        """
        selector = Selector.nsga2_selector()
        super().__init__(selector)
