from radiate.radiate import PyGeneType

class GeneType:
    """
    Class representing the type of a gene.
    """

    def __init__(self, gene_type: PyGeneType):
        self.gene_type = gene_type

    def __eq__(self, other):
        if isinstance(other, GeneType):
            return self.gene_type == other.gene_type
        return False
    
    def __repr__(self):
        return f"GeneType({self.gene_type})"

    @staticmethod
    def INT():
        return GeneType(PyGeneType.Int)

    @staticmethod
    def FLOAT():
        return GeneType(PyGeneType.Float)

    @staticmethod
    def BIT():
        return GeneType(PyGeneType.Bit)

    @staticmethod
    def CHAR():
        return GeneType(PyGeneType.Char)

    @staticmethod
    def ANY():
        return GeneType(PyGeneType.Any)


class ObjectiveType:
    """
    Class representing the type of an objective.
    """

    MIN = "min"
    MAX = "max"
