
from radiate.radiate import PyRandomProvider

def set_seed(seed: int):
    """
    Set the seed for the random number generator.
    :param seed: Seed value.
    """
    PyRandomProvider.seed(seed)