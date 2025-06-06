from radiate.radiate import PyDiversity


class DiversityBase:
    """
    Base class for diversity parameters.
    """

    def __init__(self, diversity: PyDiversity):
        """
        Initialize the diversity parameter with a PyDiversity instance.
        :param diversity: An instance of PyDiversity.
        """
        self.diversity = diversity

    def __str__(self):
        """
        Return a string representation of the diversity parameter.
        :return: String representation of the diversity parameter.
        """
        return f"Diversity(name={self.diversity.name}, args={self.diversity.args})"

    def __repr__(self):
        """
        Return a detailed string representation of the diversity parameter.
        :return: Detailed string representation of the diversity parameter.
        """
        return f"Diversity(diversity={self.diversity})"
    
    def __eq__(self, value):
        if not isinstance(value, DiversityBase):
            return False
        return self.diversity == value.diversity


class HammingDistance(DiversityBase):
    """
    Hamming Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Hamming Distance diversity parameter.
        """
        super().__init__(diversity=PyDiversity.hamming_distance())


class EuclideanDistance(DiversityBase):
    """
    Euclidean Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Euclidean Distance diversity parameter.
        """
        super().__init__(diversity=PyDiversity.euclidean_distance())
