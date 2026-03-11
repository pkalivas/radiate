class LossTypeClass(type):
    def __getitem__(cls, item):
        return cls(item)

    def __str__(cls):
        return cls.__name__


class LossType(metaclass=LossTypeClass):
    def __init__(self):
        self.name = self.__class__.__name__

    def __repr__(self) -> str:
        return self.name

    def __str__(self) -> str:
        return self.name

    def __eq__(self, value):
        if issubclass(type(value), LossType):
            return self.name == value.name
        if type(value) is LossTypeClass:
            return issubclass(value, type(self))
        else:
            return isinstance(value, type(self))


class MSE(LossType):
    """Mean Squared Error loss function."""


class MAE(LossType):
    """Mean Absolute Error loss function."""


class XEnt(LossType):
    """Cross-Entropy loss function."""


class Diff(LossType):
    """Difference loss function."""
