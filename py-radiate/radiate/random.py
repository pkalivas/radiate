from radiate.radiate import PyRandomProvider


class RandomProvider:
    def set_seed(seed: int):
        """
        Set the seed for the random number generator.
        :param seed: Seed value.
        """
        PyRandomProvider.set_seed(seed)
