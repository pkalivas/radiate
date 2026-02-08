
class DataTypeClass(type):
    def __getitem__(cls, item):
        return cls(item)
    
class DataType(metaclass=DataTypeClass):
    def __init__(self, name):
        self.name = name
    
    def __repr__(self):
        return f"DataType({self.name})"
    
INT8 = DataType("int8")
INT16 = DataType("int16")
INT32 = DataType("int32")
INT64 = DataType("int64")
INT128 = DataType("int128")
UINT8 = DataType("uint8")
UINT16 = DataType("uint16")
UINT32 = DataType("uint32")
UINT64 = DataType("uint64")
UINT128 = DataType("uint128")
FLOAT32 = DataType("float32")
FLOAT64 = DataType("float64")
USIZE = DataType("usize")
BINARY = DataType("binary")
CHAR = DataType("char")
STRING = DataType("string")
DATE = DataType("date")
VEC = DataType("vec")
STRUCT = DataType("struct")