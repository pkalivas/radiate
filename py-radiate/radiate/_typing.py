class GeneType:
    """
    Class representing the type of a gene.
    """

    FLOAT = "float"
    INT = "int"
    BIT = "bit"
    CHAR = "char"
    ANY = "any"

    ALL = [FLOAT, INT, BIT, CHAR, ANY]


class ObjectiveType:
    """
    Class representing the type of an objective.
    """

    MIN = "min"
    MAX = "max"
