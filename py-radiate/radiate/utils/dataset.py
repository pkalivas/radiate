from __future__ import annotations

from typing import TYPE_CHECKING

from .._bridge import RsObject
from .._rd import PyDataSet

if TYPE_CHECKING:
    from .._dependancies import numpy as np
    from .._typing import RdDataType


def _dtype_name(dtype: RdDataType | None) -> str | None:
    return None if dtype is None else str(dtype)


class DataSet(RsObject):
    """
    Features and labels for regression and classification, one row per sample.

    Build one from a toy dataset (``DataSet.iris()``, ``DataSet.moons(200, 0.1)``, ...),
    then shuffle, split and scale it. Each step returns a new ``DataSet``. Values are
    ``Float64`` unless ``dtype`` says otherwise. Generated datasets draw from radiate's
    random provider, so they follow its seed.
    """

    # -- Classification -------------------------------------------------------------------

    @classmethod
    def iris(cls, dtype: RdDataType | None = None) -> DataSet:
        """Fisher's iris: 150 samples, 4 features, 3 one-hot label columns. Shuffled."""
        return cls.from_rust(PyDataSet.iris(_dtype_name(dtype)))

    @classmethod
    def xor(cls, dtype: RdDataType | None = None) -> DataSet:
        """The XOR truth table: 2 binary inputs, 1 label."""
        return cls.from_rust(PyDataSet.xor(_dtype_name(dtype)))

    @classmethod
    def parity(cls, bits: int = 3, dtype: RdDataType | None = None) -> DataSet:
        """Every ``bits``-wide bit pattern, labelled 1 when an odd number of bits are set."""
        return cls.from_rust(PyDataSet.parity(bits, _dtype_name(dtype)))

    @classmethod
    def moons(
        cls, n: int = 100, noise: float = 0.1, dtype: RdDataType | None = None
    ) -> DataSet:
        """Two interleaving half circles: ``n`` samples, 2 features, a 0/1 label."""
        return cls.from_rust(PyDataSet.moons(n, noise, _dtype_name(dtype)))

    @classmethod
    def circles(
        cls, n: int = 100, noise: float = 0.1, dtype: RdDataType | None = None
    ) -> DataSet:
        """Two concentric circles: ``n`` samples, 2 features, a 0/1 label."""
        return cls.from_rust(PyDataSet.circles(n, noise, _dtype_name(dtype)))

    @classmethod
    def spirals(
        cls, n: int = 100, noise: float = 0.1, dtype: RdDataType | None = None
    ) -> DataSet:
        """Two intertwined spirals: ``n`` samples, 2 features, a 0/1 label."""
        return cls.from_rust(PyDataSet.spirals(n, noise, _dtype_name(dtype)))

    # -- Regression -----------------------------------------------------------------------

    @classmethod
    def cubic(cls, dtype: RdDataType | None = None) -> DataSet:
        """``4x³ - 3x² + x`` at 20 evenly spaced points."""
        return cls.from_rust(PyDataSet.cubic(_dtype_name(dtype)))

    @classmethod
    def koza1(cls, dtype: RdDataType | None = None) -> DataSet:
        """Koza-1: ``x⁴ + x³ + x² + x`` at 20 random points in [-1, 1)."""
        return cls.from_rust(PyDataSet.koza1(_dtype_name(dtype)))

    @classmethod
    def nguyen7(cls, dtype: RdDataType | None = None) -> DataSet:
        """Nguyen-7: ``ln(x + 1) + ln(x² + 1)`` at 20 random points in [0, 2)."""
        return cls.from_rust(PyDataSet.nguyen7(_dtype_name(dtype)))

    @classmethod
    def pagie1(cls, dtype: RdDataType | None = None) -> DataSet:
        """Pagie-1: ``1/(1 + x⁻⁴) + 1/(1 + y⁻⁴)`` on a 26 x 26 grid over [-5, 5]."""
        return cls.from_rust(PyDataSet.pagie1(_dtype_name(dtype)))

    # -- Sequences ------------------------------------------------------------------------

    @classmethod
    def simple_memory(cls, n: int = 10, dtype: RdDataType | None = None) -> DataSet:
        return cls.from_rust(PyDataSet.simple_memory(n, _dtype_name(dtype)))

    # -- Data -----------------------------------------------------------------------------
    @property
    def features(self) -> np.ndarray:
        """The features as a ``(samples, features)`` array in the dataset's dtype."""
        return self.__backend__().features()

    @property
    def labels(self) -> np.ndarray:
        """The labels as a ``(samples, labels)`` array in the dataset's dtype."""
        return self.__backend__().labels()

    @property
    def shape(self) -> tuple[int, int, int]:
        """``(samples, features, labels)``."""
        return self.__backend__().shape()

    def __len__(self) -> int:
        return self.shape[0]

    # -- Transforms -----------------------------------------------------------------------

    def pair(self) -> tuple[np.ndarray, np.ndarray]:
        """Returns the features and labels as a tuple."""
        return self.features, self.labels

    def shuffle(self) -> DataSet:
        """A copy with the rows in random order."""
        return DataSet.from_rust(self.__backend__().shuffle())

    def split(self, split_pct: float) -> tuple[DataSet, DataSet]:
        """Splits the rows into ``(train, test)``, in their current order."""
        train, test = self.__backend__().split(split_pct)
        return DataSet.from_rust(train), DataSet.from_rust(test)

    def standardize(self) -> DataSet:
        """A copy with each feature column scaled to mean 0, standard deviation 1."""
        return DataSet.from_rust(self.__backend__().standardize())

    def normalize(self) -> DataSet:
        """A copy with each feature column scaled to [0, 1]."""
        return DataSet.from_rust(self.__backend__().normalize())
