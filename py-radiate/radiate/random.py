from radiate.radiate import PyRandomProvider


class RandomProvider:
    def set_seed(seed: int):
        """
        Set the seed for the random number generator.
        :param seed: Seed value.
        """
        PyRandomProvider.set_seed(seed)

    def randint(min: int, max: int) -> int:
        """
        Generate a random integer in the range [min, max).
        :param min: Minimum value (inclusive).
        :param max: Maximum value (exclusive).
        :return: Random integer.
        """
        return PyRandomProvider.random_int(min, max)

    def randfloat(min: float = 0.0, max: float = 1.0) -> float:
        """
        Generate a random float in the range [min, max).
        :param min: Minimum value (inclusive).
        :param max: Maximum value (exclusive).
        :return: Random float.
        """
        return PyRandomProvider.random_float(min, max)
