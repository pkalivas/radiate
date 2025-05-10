from typing import Dict, Any
from .param import EngineParam


class Selector(EngineParam):
    def __init__(self, name: str, args: Dict[str, Any] = None):
        """
        Initialize the selector with name and additional parameters.
        :param name: Name of the selector.
        :param args: Additional parameters for the selector.
        """
        super().__init__(name=name, args=args)


class TournamentSelector(Selector):
    def __init__(self, k: int = 3):
        """
        Initialize the tournament selector with tournament size.
        :param k: Tournament size.
        """
        super().__init__(name="tournament", args={"k": str(k)})


class RouletteSelector(Selector):
    def __init__(self):
        """
        Initialize the roulette selector.
        """
        super().__init__(name="roulette")


class RankSelector(Selector):
    def __init__(self):
        """
        Initialize the rank selector.
        """
        super().__init__(name="rank")


class ElitismSelector(Selector):
    def __init__(self):
        """
        Initialize the elitism selector.
        """
        super().__init__(name="elitism")


class BoltzmannSelector(Selector):
    def __init__(self, temp: float = 1.0):
        """
        Initialize the Boltzmann selector with temperature.
        :param temp: Temperature for the Boltzmann selector.
        """
        super().__init__(name="boltzmann", args={"temp": str(temp)})


class StocasticSamplingSelector(Selector):
    def __init__(self):
        """
        Initialize the stochastic sampling selector.
        """
        super().__init__(name="stocastic_universal_sampling")


class LinearRankSelector(Selector):
    def __init__(self, pressure: float = 0.5):
        """
        Initialize the linear rank selector.
        """
        super().__init__(name="linear_rank", args={"pressure": str(pressure)})


class NSGA2Selector(Selector):
    def __init__(self):
        """
        Initialize the NSGA2 selector.
        """
        super().__init__(
            name="nsga2",
        )
