from radiate.radiate import _get_dtype_max, _get_dtype_min


class DataTypeClass(type):
    def __getitem__(cls, item):
        return cls(item)
    


class DataType(metaclass=DataTypeClass):
    def __init__(self):
        self.name = self.__class__.__name__

    def __repr__(self) -> str:
        return self.name
    
    def __eq__(self, value):
        if issubclass(type(value), DataType):
            return self.name == value.name
        if type(value) is DataTypeClass:
            return issubclass(value, type(self))
        else:
            return isinstance(value, type(self))


class NumericType(DataType):
    @classmethod
    def max(cls) -> int | float:
        return _get_dtype_max(cls.__name__)

    @classmethod
    def min(cls) -> int | float:
        return _get_dtype_min(cls.__name__)


class IntegerType(NumericType):
    """Integer data type."""


class FloatType(NumericType):
    """Floating-point data type."""


"""
Unsigned Integer Types
-----------------------
"""


class UInt8(IntegerType):
    """Unsigned 8-bit integer."""


class UInt16(IntegerType):
    """Unsigned 16-bit integer."""


class UInt32(IntegerType):
    """Unsigned 32-bit integer."""


class UInt64(IntegerType):
    """Unsigned 64-bit integer."""


class UInt128(IntegerType):
    """Unsigned 128-bit integer."""


"""
Signed Integer Types
-----------------------
"""


class Int8(IntegerType):
    """Signed 8-bit integer."""


class Int16(IntegerType):
    """Signed 16-bit integer."""


class Int32(IntegerType):
    """Signed 32-bit integer."""


class Int64(IntegerType):
    """Signed 64-bit integer."""


class Int128(IntegerType):
    """Signed 128-bit integer."""


"""
Floating-Point Types
-----------------------
"""


class Float32(FloatType):
    """32-bit floating-point number."""


class Float64(FloatType):
    """64-bit floating-point number."""


"""
Boolean Type
-----------------------
"""


class Bool(DataType):
    """Boolean data type."""


"""
Usize Type
-----------------------
"""


class Usize(IntegerType):
    """Unsigned integer type with the same number of bits as the platform's pointer type."""


"""
Struct Type
-----------------------
"""


class Struct(DataType):
    """Struct data type."""


"""
Utility functions
"""


def dtype_from_str(dtype_str: str) -> DataType | None:
    """
    Convert a string representation of a data type to a DataType instance.

    :param dtype_str: The string representation of the data type.
    :return: A DataType instance corresponding to the string, or None if the string is not recognized.
    """
    match dtype_str.lower():
        case "uint8":
            return UInt8()
        case "uint16":
            return UInt16()
        case "uint32":
            return UInt32()
        case "uint64":
            return UInt64()
        case "uint128":
            return UInt128()
        case "int8":
            return Int8()
        case "int16":
            return Int16()
        case "int32":
            return Int32()
        case "int64":
            return Int64()
        case "int128":
            return Int128()
        case "float32":
            return Float32()
        case "float64":
            return Float64()
        case "bool":
            return Bool()
        case "usize":
            return Usize()
        case "struct":
            return Struct()
        case _:
            return None
