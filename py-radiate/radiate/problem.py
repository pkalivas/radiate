import abc

from typing import Callable, Any
from .codec import CodecBase
from .genome import Genotype

class ProblemBase(abc.ABC):
    """A class representing a problem to be solved using evolutionary algorithms."""

    def __init__(self, codec: CodecBase | None = None, **kwargs):
        """
        Initializes the Problem instance.

        :param codec: An instance of CodecBase used for encoding/decoding genotypes.
        :param kwargs: Additional keyword arguments for problem configuration.
        """
        if codec is None:
            raise ValueError("codec must be provided")
        
        self.codec = codec
        self.args = kwargs

    @abc.abstractmethod
    def evaluate(self, genotype: "Genotype") -> Any:
        """
        Evaluates the fitness of a given genotype.

        :param genotype: An instance of Genotype to be evaluated.
        :return: The fitness score of the genotype.
        """
        pass


class Regression(ProblemBase):
    """A class representing a regression problem."""

    def __init__(self, codec: CodecBase, data: list[tuple[list[float], float]], **kwargs):
        """
        Initializes the Regression problem instance.

        :param codec: An instance of CodecBase used for encoding/decoding genotypes.
        :param data: A list of tuples where each tuple contains input features and the corresponding target value.
        :param kwargs: Additional keyword arguments for problem configuration.
        """
        super().__init__(codec, **kwargs)
        self.data = data

    def evaluate(self, genotype: "Genotype") -> float:
        """
        Evaluates the fitness of a given genotype using Mean Squared Error (MSE).

        :param genotype: An instance of Genotype to be evaluated.
        :return: The Mean Squared Error (MSE) of the genotype on the provided data.
        """
        # Decode the genotype to obtain the model parameters
        model = self.codec.decode(genotype)

        # Calculate Mean Squared Error (MSE)
        mse = 0.0
        for inputs, target in self.data:
            prediction = model.predict(inputs)
            mse += (prediction - target) ** 2
        mse /= len(self.data)

        return mse