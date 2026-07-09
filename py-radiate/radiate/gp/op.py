from __future__ import annotations

from collections.abc import Callable
from typing import Any

from radiate.radiate import _all_ops, _create_op

from .._bridge import LazyRsObject
from .._typing import RdDataType
from ..dsl.dtype import Float64


def _op_factory(name: str):
    def op(dtype: RdDataType | None = None) -> Op:
        return Op(name, dtype=dtype)

    return staticmethod(op)


class OpBuilder:
    def __init__(self, dtype: RdDataType, **kwargs):
        self.dtype = dtype
        self.ops = {}

        for key, op in kwargs.items():
            if op is not None:
                if isinstance(op, Op):
                    op._dtype = dtype
                    self.ops[key] = [op]
                elif isinstance(op, (list, tuple)):
                    for o in op:
                        o._dtype = dtype
                    self.ops[key] = list(op)


class Op(LazyRsObject):
    _name: str
    _dtype: RdDataType = Float64
    _args: dict[str, object]

    def __init__(self, name: str, dtype: RdDataType | None = None, **kwargs):
        super().__init__()
        self._name = name
        self._dtype = dtype if dtype is not None else Float64
        self._args = kwargs

    def _initialize(self) -> Callable[[], Any]:
        def build():
            return _create_op(self._name, str(self._dtype), self._args)

        return build

    def eval(self, *args):
        return self.__backend__().eval(list(args))

    def name(self) -> str:
        return self.__backend__().name

    @staticmethod
    def var(idx: int = 0, dtype: RdDataType | None = None) -> Op:
        return Op("var", dtype=dtype, index=idx)

    @staticmethod
    def const(value: float, dtype: RdDataType | None = None) -> Op:
        return Op("constant", dtype=dtype, value=value)

    add = _op_factory("add")
    sub = _op_factory("sub")
    mul = _op_factory("mul")
    div = _op_factory("div")
    sigmoid = _op_factory("sigmoid")
    weight = _op_factory("weight")
    relu = _op_factory("relu")
    tanh = _op_factory("tanh")
    linear = _op_factory("linear")
    sum = _op_factory("sum")
    prod = _op_factory("prod")
    diff = _op_factory("diff")
    pow = _op_factory("pow")
    log = _op_factory("log")
    sin = _op_factory("sin")
    cos = _op_factory("cos")
    identity = _op_factory("identity")
    neg = _op_factory("neg")
    sqrt = _op_factory("sqrt")
    abs = _op_factory("abs")
    exp = _op_factory("exp")
    tan = _op_factory("tan")
    ceil = _op_factory("ceil")
    floor = _op_factory("floor")
    max = _op_factory("max")
    min = _op_factory("min")
    leaky_relu = _op_factory("leaky_relu")
    elu = _op_factory("elu")
    mish = _op_factory("mish")
    swish = _op_factory("swish")
    softplus = _op_factory("softplus")

    @staticmethod
    def default_vertex_ops(dtype: RdDataType | None = None) -> list[Op]:
        dtype = dtype if dtype is not None else Float64
        return [
            Op.add(dtype=dtype),
            Op.sub(dtype=dtype),
            Op.mul(dtype=dtype),
            Op.div(dtype=dtype),
            Op.sin(dtype=dtype),
            Op.cos(dtype=dtype),
            Op.tanh(dtype=dtype),
            Op.relu(dtype=dtype),
            Op.linear(dtype=dtype),
        ]

    @staticmethod
    def default_edge_ops(dtype: RdDataType | None = None) -> list[Op]:
        dtype = dtype if dtype is not None else Float64
        return [Op.weight(dtype=dtype)]

    @staticmethod
    def all_ops(dtype: RdDataType | None = None) -> list[Op]:
        dtype_str = str(dtype) if dtype is not None else str(Float64)
        return [Op.from_rust(op) for op in _all_ops(dtype_str)]
