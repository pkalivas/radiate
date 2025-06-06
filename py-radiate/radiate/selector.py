from radiate.radiate import PySelector


class SelectorBase:
    def __init__(self, selector: PySelector):
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

    def __eq__(self, value):
        if not isinstance(value, SelectorBase):
            return False
        return self.selector == value.selector


class TournamentSelector(SelectorBase):
    def __init__(self, k: int = 3):
        """
        Initialize the tournament selector with tournament size.
        :param k: Tournament size.
        """
        selector = PySelector.tournament_selector(tournament_size=k)
        super().__init__(selector)


class RouletteSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the roulette selector.
        """
        selector = PySelector.roulette_wheel_selector()
        super().__init__(selector)


class RankSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the rank selector.
        """
        selector = PySelector.rank_selector()
        super().__init__(selector)


class EliteSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the elite selector.
        """
        selector = PySelector.elite_selector()
        super().__init__(selector)


class BoltzmannSelector(SelectorBase):
    def __init__(self, temp: float = 1.0):
        """
        Initialize the Boltzmann selector with temperature.
        :param temp: Temperature for the Boltzmann selector.
        """
        selector = PySelector.boltzmann_selector(temp=temp)
        super().__init__(selector)


class StochasticSamplingSelector(SelectorBase):
    def __init__(self):
        """
        Initialize the stochastic sampling selector.
        """
        selector = PySelector.stochastic_sampling_selector()
        super().__init__(selector)


class LinearRankSelector(SelectorBase):
    def __init__(self, pressure: float = 0.5):
        """
        Initialize the linear rank selector.
        :param pressure: Pressure for the linear rank selector.
        """
        selector = PySelector.linear_rank_selector(pressure=pressure)
        super().__init__(selector)


class NSGA2Selector(SelectorBase):
    def __init__(self):
        """
        Initialize the NSGA2 selector.
        """
        selector = PySelector.nsga2_selector()
        super().__init__(selector)
