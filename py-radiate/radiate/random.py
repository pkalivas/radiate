from radiate.radiate import PyRandomProvider


class RandomProvider:
    def seed(seed: int):
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

    def float(min: float = 0.0, max: float = 1.0) -> float:
        """
        Generate a random float in the range [min, max).
        :param min: Minimum value (inclusive).
        :param max: Maximum value (exclusive).
        :return: Random float.
        """
        return PyRandomProvider.random_float(min, max)

    def sample(data: list, count: int) -> list:
        """
        Randomly sample elements from a list.
        :param data: List of elements to sample from.
        :param count: Number of elements to sample.
        :return: List of sampled elements.
        """
        return PyRandomProvider.sample(data, count)
