from typing import TypeAlias, Any, Union

# GeneType: TypeAlias = Union[type[float], type[int], type[bool], type[str], type[Any]]

class GeneType:
    """
    Class representing the type of a gene.
    """
    FLOAT = 'float'
    INT = 'int'
    BOOL = 'bool'
    CHAR = 'char'
    ANY = 'any'

    ALL = [FLOAT, INT, BOOL, CHAR, ANY]