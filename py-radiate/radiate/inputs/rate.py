from radiate.wrapper import PyObject
from radiate.radiate import PyRate


class Rate(PyObject[PyRate]):
    CYCLICAL = "cyclical"
    FIXED = "fixed"
    LINEAR = "linear"
    STEPWISE = "stepwise"
    EXPONENTIAL = "exponential"

    def __init__(
        self,
        rate_type: str,
        start: float = 0.0,
        end: float = 1.0,
        duration: int = 1,
        shape: str = "sine",
        warmup_duration: int = 0,
        half_life: int = 1,
        peak: float = 1.0,
        steps: list[tuple[int, float]] | None = None,
    ):
        if rate_type == self.FIXED:
            self._pyobj = PyRate.fixed(start)
        elif rate_type == self.LINEAR:
            self._pyobj = PyRate.linear(start, end, duration)
        elif rate_type == self.CYCLICAL:
            self._pyobj = PyRate.cyclical(start, end, duration, shape)
        elif rate_type == self.STEPWISE:
            if steps is None:
                raise ValueError("Steps must be provided for step rate type.")
            self._pyobj = PyRate.stepwise(steps)
        elif rate_type == self.EXPONENTIAL:
            self._pyobj = PyRate.exponential(start, end, duration)
        else:
            raise ValueError(f"Unknown rate type: {rate_type}")

    def value(self, index: int) -> float:
        """
        Get the rate value at a specific index.

        :param index: The index to get the rate value for.
        :return: The rate value at the specified index.
        """
        return self._pyobj.value(index)

    def fixed(rate: float):
        return Rate(Rate.FIXED, rate)

    def linear(start: float, end: float, duration: int):
        return Rate(Rate.LINEAR, start, end, duration)
    
    def sine(min: float, max: float, periods: int):
        return Rate(Rate.CYCLICAL, min, max, periods, "sine")

    def triangular(min: float, max: float, periods: int):
        return Rate(Rate.CYCLICAL, min, max, periods, "triangular")

    def stepwise(steps: list[tuple[int, float]]):
        return Rate(Rate.STEPWISE, steps=steps)

    def exp(start: float, end: float, half_life: int):
        return Rate(Rate.EXPONENTIAL, start, end, half_life)


def fixed(rate: float):
    return Rate(Rate.FIXED, rate)


def sine(min: float, max: float, periods: int):
    return Rate(Rate.CYCLICAL, min, max, periods, "sine")


def triangular(min: float, max: float, periods: int):
    return Rate(Rate.CYCLICAL, min, max, periods, "triangular")


def linear(start: float, end: float, duration: int):
    return Rate(Rate.LINEAR, start, end, duration)


def stepwise(steps: list[tuple[int, float]]):
    return Rate(Rate.STEPWISE, steps=steps)


def exp(start: float, end: float, half_life: int):
    return Rate(Rate.EXPONENTIAL, start, end, half_life)
