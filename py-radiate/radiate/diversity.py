

from radiate.radiate import PyDiversity

class Diversity:
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
    

class HammingDistance(Diversity):
    """
    Hamming Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Hamming Distance diversity parameter.
        """
        super().__init__(diversity=PyDiversity.hamming_distance())


class EuclideanDistance(Diversity):
    """
    Euclidean Distance diversity parameter.
    """

    def __init__(self):
        """
        Initialize the Euclidean Distance diversity parameter.
        """
        super().__init__(diversity=PyDiversity.euclidean_distance())

# from typing import List
# from .param import EngineParam
# from ._typing import GeneType

# class Diversity(EngineParam):
#     """
#     Class to hold diversity parameters.
#     """

#     gene_typess: List[GeneType] = []
#     name: str = None

#     def __init__(self, name: str, args: dict = None, gene_types: List[GeneType] = None):
#         """
#         Initialize the diversity parameters.
#         :param name: Name of the diversity parameter.
#         :param args: Arguments for the diversity parameter.
#         """
#         super().__init__(name=name, args={k: str(v) for k, v in args.items()})
#         self.gene_types = gene_types or self.gene_types
    
#     def __repr__(self):
#         return f"Diversity(name={self.name}, args={self.args}, gene_types={self.gene_types})"

#     def __str__(self):
#         return f"Diversity(name={self.name}, args={self.args}, gene_types={self.gene_types})"
    
#     def is_valid(self, gene_type: GeneType):
#         """
#         Validate the gene type.
#         :param gene_type: Gene type to validate.
#         :return: True if the gene type is valid, False otherwise.
#         """
#         if not isinstance(gene_type, GeneType):
#             raise TypeError(
#                 f"Gene type {type(gene_type)} is not supported. Expected GeneType."
#             )
#         return gene_type in self.gene_types
    

# class Hammingdistance(Diversity):
#     """
#     Hamming Distance parameter.
#     """

#     gene_types = [GeneType.BIT(), GeneType.CHAR(), GeneType.INT(), GeneType.FLOAT()]
#     name = "hamming_distance"

#     def __init__(self):
#         super().__init__(name=self.name, args={}, gene_types=self.gene_types)


# class EuclideanDistance(Diversity):
#     """
#     Euclidean Distance parameter.
#     """

#     gene_types = [GeneType.FLOAT(), GeneType.INT()]
#     name = "euclidean_distance"

#     def __init__(self):
#         super().__init__(name=self.name, args={}, gene_types=self.gene_types)

