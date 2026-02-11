from __future__ import annotations

from radiate.radiate import _get_dtype_max, _get_dtype_min
from inspect import isclass
from typing import Iterator


class Field:
    """
    Definition of a single field within a `Struct` DataType.

    Parameters
    ----------
    name
        The name of the field within its parent `Struct`.
    dtype
        The `DataType` of the field's values.
    """

    name: str
    dtype: DataType | DataTypeClass

    def __init__(self, name: str, dtype: DataType | DataTypeClass) -> None:
        self.name = name
        self.dtype = dtype

    def __eq__(self, other) -> bool:  # type: ignore[override]
        return (self.name == other.name) & (self.dtype == other.dtype)

    def __hash__(self) -> int:
        return hash((self.name, self.dtype))

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.name!r}, {self.dtype})"


class DataTypeClass(type):
    def __getitem__(cls, item):
        return cls(item)

    def __str__(cls):
        return cls.__name__


class DataType(metaclass=DataTypeClass):
    def __init__(self):
        self.name = self.__class__.__name__

    def __repr__(self) -> str:
        return self.name

    def __str__(self) -> str:
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

    @classmethod
    def is_int(cls) -> bool:
        return issubclass(cls, IntegerType)

    @classmethod
    def is_float(cls) -> bool:
        return issubclass(cls, FloatType)


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


class Boolean(DataType):
    """Boolean data type."""


"""
Usize Type
-----------------------
"""


class String(DataType):
    """UTF-8 encoded string type."""


class Char(DataType):
    """Single character string type."""


class Null(DataType):
    """Null type, representing the absence of a value."""


class Op32(DataType):
    """Data type representing a 32-bit operation, used for graph and tree nodes."""


class NestedType(DataType):
    """Base class for nested data types."""


class Struct(NestedType):
    """
        Struct data type, representing a collection of named fields.
        Parameters
    ----------
    fields    A list of `Field` instances defining the structure of the `Struct`.
    Examples
    --------
    >>> person_dtype = Struct([
    ...     Field("name", String),
    ...     Field("age", Int32),
    ...     Field("is_student", Bool),
    ... ])
    >>> person_dtype
    Struct({'name': String, 'age': Int32, 'is_student': Bool})
    """

    fields: list[Field]

    def __init__(self, fields: list[Field]) -> None:
        self.fields = list(fields)

    def __eq__(self, other) -> bool:  # type: ignore[override]
        if isclass(other) and issubclass(other, Struct):
            return True
        elif isinstance(other, Struct):
            return self.fields == other.fields
        else:
            return False

    def __hash__(self) -> int:
        return hash((self.__class__, tuple(self.fields)))

    def __iter__(self) -> Iterator[tuple[str, DataType | DataTypeClass]]:
        for fld in self.fields:
            yield fld.name, fld.dtype

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({dict(self)})"

    def __str__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({dict(self)})"


class List(NestedType):
    """
        List data type, representing a homogeneous collection of values.
        Parameters
    ----------
    inner    The `DataType` of the values contained within the list. If not specified, the list is considered to be of
      an unspecified inner type, and will compare as equal to any other List type (eg: List[Int32] == List == List[Float64]).
    Examples
    --------
    >>> int_list_dtype = List(Int32)
    >>> int_list_dtype
    List(Int32)
    >>> int_list_dtype == List(Int32)
    True
    >>> int_list_dtype == List(Float64)
    False
    >>> int_list_dtype == List
    True
    >>> List == List(Int32)
    True
    """

    inner: DataTypeClass | DataType

    def __init__(self, inner: DataTypeClass | DataType) -> None:
        # self.inner = polars.datatypes.parse_into_dtype(inner)
        self.inner = inner

    def __eq__(self, other: DataTypeClass | DataType) -> bool:  # type: ignore[override]
        # allow comparing object instances to class
        if type(other) is DataTypeClass and issubclass(other, List):
            return True
        elif isinstance(other, List):
            return self.inner == other.inner
        else:
            return False

    def __hash__(self) -> int:
        return hash((self.__class__, self.inner))

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.inner!r})"

    def __str__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.inner})"


class Node(NestedType):
    """
    Node data type, representing a graph or tree node with an associated operation.
    Parameters
    ----------
    inner    The `DataType` of the operation associated with the node. Typically this will be `Op32`, which represents a 32-bit operation.
    Examples
    --------
    >>> node_dtype = Node(Op32)
    >>> node_dtype
    Node(Op32)
    >>> node_dtype == Node(Op32)
    True
    >>> node_dtype == Node(String)
    False
    >>> node_dtype == Node
    True
    >>> Node == node_dtype
    True
    """

    inner: DataTypeClass | DataType

    def __init__(self, inner: DataTypeClass | DataType) -> None:
        self.inner = inner

    def __eq__(self, other: DataTypeClass | DataType) -> bool:  # type: ignore[override]
        # allow comparing object instances to class
        if type(other) is DataTypeClass and issubclass(other, Node):
            return True
        elif isinstance(other, Node):
            return self.inner == other.inner
        else:
            return False

    def __hash__(self) -> int:
        return hash((self.__class__, self.inner))

    def __repr__(self) -> str:  
        class_name = self.__class__.__name__
        return f"{class_name}({self.inner!r})"

    def __str__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.inner})"
