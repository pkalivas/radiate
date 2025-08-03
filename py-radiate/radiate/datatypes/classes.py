class DataTypeClass(type):
    """Metaclass for DataType classes."""

    def __repr__(cls) -> str:
        return cls.__name__

    def __eq__(cls, other) -> bool:
        if isinstance(other, DataTypeClass):
            return cls is other
        return False

    def __hash__(cls) -> int:
        return hash(cls.__name__)


class DataType(metaclass=DataTypeClass):
    """Base class for all fitness function data types."""

    def __repr__(self) -> str:
        return self.__class__.__name__

    def __eq__(self, other) -> bool:
        return isinstance(other, type(self))

    def __hash__(self) -> int:
        return hash(self.__class__)

    @classmethod
    def base_type(cls):
        """Return this DataType's fundamental/root type class."""
        return cls

    @classmethod
    def is_scalar(cls) -> bool:
        """Check whether the data type is a scalar type."""
        return not issubclass(cls, ArrayType) and not issubclass(cls, MatrixType)

    @classmethod
    def is_nested(cls) -> bool:
        """Check whether the data type is a wrapper type."""
        return issubclass(cls, ArrayType) or issubclass(cls, MatrixType)

    @classmethod
    def is_numeric(cls) -> bool:
        """Check whether the data type is a numeric type."""
        return issubclass(cls, NumericType)

    @classmethod
    def is_integer(cls) -> bool:
        """Check whether the data type is an integer type."""
        return issubclass(cls, IntegerType)

    @classmethod
    def is_float(cls) -> bool:
        """Check whether the data type is a floating point type."""
        return issubclass(cls, FloatType)

    @classmethod
    def is_char(cls) -> bool:
        """Check whether the data type is a character type."""
        return issubclass(cls, Char)

    @classmethod
    def is_bool(cls) -> bool:
        """Check whether the data type is a boolean type."""
        return issubclass(cls, Bool)

    @classmethod
    def is_array(cls) -> bool:
        """Check whether the data type is an array type."""
        return issubclass(cls, ArrayType)

    @classmethod
    def is_matrix(cls) -> bool:
        """Check whether the data type is a matrix type."""
        return issubclass(cls, MatrixType)


# Generic base types
class ScalarType(DataType):
    """Base class for scalar data types."""


class NumericType(ScalarType):
    """Base class for numeric data types."""


class IntegerType(NumericType):
    """Base class for integer data types."""


class FloatType(NumericType):
    """Base class for float data types."""


class ArrayType(DataType):
    """Base class for array data types."""


class MatrixType(DataType):
    """Base class for matrix data types."""


# Scalar types
class Int32(IntegerType):
    """32-bit signed integer type."""


class Int64(IntegerType):
    """64-bit signed integer type."""


class Float32(FloatType):
    """32-bit floating point type."""


class Float64(FloatType):
    """64-bit floating point type."""


class Bool(DataType):
    """Boolean type."""


class Char(DataType):
    """Character type."""


# Array types (1D)
class Int32Array(ArrayType, IntegerType):
    """32-bit signed integer array type."""


class Int64Array(ArrayType, IntegerType):
    """64-bit signed integer array type."""


class Float32Array(ArrayType, FloatType):
    """32-bit floating point array type."""


class Float64Array(ArrayType, FloatType):
    """64-bit floating point array type."""


class BoolArray(ArrayType):
    """Boolean array type."""


class CharArray(ArrayType):
    """Character array type."""


# Matrix types (2D)
class Int32Matrix(MatrixType, IntegerType):
    """32-bit signed integer matrix type."""


class Int64Matrix(MatrixType, IntegerType):
    """64-bit signed integer matrix type."""


class Float32Matrix(MatrixType, FloatType):
    """32-bit floating point matrix type."""


class Float64Matrix(MatrixType, FloatType):
    """64-bit floating point matrix type."""


class BoolMatrix(MatrixType):
    """Boolean matrix type."""


class CharMatrix(MatrixType):
    """Character matrix type."""


