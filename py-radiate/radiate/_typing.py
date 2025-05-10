class GeneType:
    """
    Class representing the type of a gene.
    """

    FLOAT = "float"
    INT = "int"
    BOOL = "bool"
    CHAR = "char"
    ANY = "any"

    ALL = [FLOAT, INT, BOOL, CHAR, ANY]


class ObjectiveType:
    """
    Class representing the type of an objective.
    """

    MIN = "min"
    MAX = "max"
