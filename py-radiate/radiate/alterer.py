from ._typing import GeneType
from .param import EngineParam
from typing import Dict, List


class Alterer(EngineParam):
    """
    Base class for all alterers.
    """
    gene_types: List[GeneType] = []
    name: str = None

    def __init__(self, name: str, args: Dict[str, any] = None, gene_types: List[GeneType] = None):
        """
        Initialize the alterer.
        :param name: Name of the alterer.
        :param args: Arguments for the alterer.
        :param gene_types: List of gene types for the alterer.
        """
        super().__init__(name=name, args={k: str(v) for k, v in args.items()})
        self.gene_types = gene_types or self.gene_types
        

    def __getattr__(self, name):
        """
        Get the value of an attribute.
        :param name: Name of the attribute.
        :return: Value of the attribute.
        """
        if name in self.args:
            return self.args[name]
        if name in self.__dict__:
            return self.__dict__[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
    
    def is_valid(self, gene_type: GeneType):
        """
        Validate the gene type.
        :param gene_type: Gene type to validate.
        :return: True if the gene type is valid, False otherwise.
        """
        return gene_type in self.gene_types
    

class BlendCrossover(Alterer):
    """
    Blend Crossover alterer.
    """
    gene_types = [GeneType.FLOAT, GeneType.INT]
    name = 'blend_crossover'

    def __init__(self, rate: float = 0.1, alpha: float = 0.5):
        """
        Initialize the blend crossover alterer.
        :param alpha: Alpha value for the blend crossover.
        :param rate: Rate of crossover.
        """
        super().__init__(name=self.name, args={'rate': rate, 'alpha': alpha}, gene_types=self.gene_types)



class IntermediateCrossover(Alterer):
    """
    Intermediate Crossover alterer.
    """
    gene_types = [GeneType.FLOAT, GeneType.INT]
    name = 'intermediate_crossover'

    def __init__(self, rate: float=0.1, alpha: float = 0.5):
        """
        Initialize the intermediate crossover alterer.
        :param alpha: Alpha value for the intermediate crossover.
        """
        super().__init__(name=self.name, args={'rate': rate, 'alpha': alpha}, gene_types=self.gene_types)
        

class MultiPointCrossover(Alterer):
    """
    Multi Point Crossover alterer.
    """
    gene_types = GeneType.ALL
    name = 'multi_point_crossover'

    def __init__(self, rate: float=0.1, num_points: int=2):
        """
        Initialize the multi point crossover alterer.
        :param alpha: Alpha value for the multi point crossover.
        """
        super().__init__(name=self.name, args={'rate': rate, 'num_points': num_points}, gene_types=self.gene_types)


class UniformCrossover(Alterer):
    """
    Uniform Crossover alterer.
    """
    gene_types = GeneType.ALL
    name = 'uniform_crossover'

    def __init__(self, rate: float=0.5):
        """
        Initialize the uniform crossover alterer.
        :param alpha: Alpha value for the uniform crossover.
        """
        super().__init__(name=self.name, args={'rate': rate}, gene_types=self.gene_types)


class UniformMutator(Alterer):
    """
    Uniform Mutator alterer.
    """
    gene_types = GeneType.ALL
    name = 'uniform_mutator'

    def __init__(self, rate: float=0.1):
        """
        Initialize the uniform mutator alterer.
        :param rate: Rate of mutation.
        """
        super().__init__(name=self.name, args={'rate': rate}, gene_types=self.gene_types)


class ArithmeticMutator(Alterer):
    """
    Arithmetic Mutator alterer.
    """
    gene_types = [GeneType.FLOAT, GeneType.INT]
    name = 'arithmetic_mutator'

    def __init__(self, rate: float = 0.1):
        """
        Initialize the arithmetic mutator alterer.
        :param alpha: Alpha value for the arithmetic mutator.
        """
        super().__init__(name=self.name, args={'rate': rate}, gene_types=self.gene_types)