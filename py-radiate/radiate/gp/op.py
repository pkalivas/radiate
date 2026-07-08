from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

from radiate.radiate import _all_ops, _create_op

from .._bridge import LazyRsObject
from .._typing import AtLeastOne, RdDataType
from ..dsl.dtype import Float64


def _op_factory(name: str):
    def op(dtype: RdDataType | None = None) -> Op:
        return Op(name, dtype=dtype)

    return staticmethod(op)


@dataclass(frozen=True, slots=True)
class OpsConfig:
    vertex: AtLeastOne[Op] | None = None
    edge: AtLeastOne[Op] | None = None
    output: AtLeastOne[Op] | None = None
    leaf: AtLeastOne[Op] | None = None
    root: AtLeastOne[Op] | None = None
    values: dict[str, AtLeastOne[Op]] | None = None

    def build_ops_map(
        self, input_size: int, dtype: RdDataType, fill_invalid: bool = False
    ) -> dict[str, list[Op]]:
        def inner():
            base = {}
            for i in range(input_size):
                base.setdefault("input", []).append(Op.var(i, dtype=dtype))
                base.setdefault("leaf", []).append(Op.var(i, dtype=dtype))

            if self.values is not None:
                merged = dict(self.values) | base

                result = merged | base
                {
                    node_type: [op.__backend__() for op in ops]
                    for node_type, ops in map(
                        lambda pair: (
                            pair[0],
                            [pair[1]] if isinstance(pair[1], Op) else list(pair[1]),
                        ),
                        result.items(),
                    )
                }

            ops_map = dict(base)
            if self.vertex is not None:
                ops_map["vertex"] = (
                    [self.vertex] if isinstance(self.vertex, Op) else list(self.vertex)
                )
            if self.edge is not None:
                ops_map["edge"] = (
                    [self.edge] if isinstance(self.edge, Op) else list(self.edge)
                )
            if self.output is not None:
                ops_map["output"] = (
                    [self.output] if isinstance(self.output, Op) else list(self.output)
                )
            if self.leaf is not None:
                ops_map["leaf"] = (
                    [self.leaf] if isinstance(self.leaf, Op) else list(self.leaf)
                )
            if self.root is not None:
                ops_map["root"] = (
                    [self.root] if isinstance(self.root, Op) else list(self.root)
                )

            return {
                node_type: [op.__backend__() for op in ops]
                for node_type, ops in ops_map.items()
            }

        ops_map = inner()

        if fill_invalid:
            if "vertex" not in ops_map:
                ops_map["vertex"] = [
                    op.__backend__() for op in Op.default_vertex_ops(dtype=Float64)
                ]
            if "edge" not in ops_map:
                ops_map["edge"] = [
                    op.__backend__() for op in Op.default_edge_ops(dtype=Float64)
                ]
            if "output" not in ops_map:
                ops_map["output"] = [Op.linear(dtype=Float64).__backend__()]

        return ops_map


class OpBuilder:
    def __init__(self, dtype: RdDataType, **kwargs):
        self.dtype = dtype
        self.ops_map = {}

        for key, op in kwargs.items():
            if op is not None:
                if isinstance(op, Op):
                    op._dtype = dtype
                    self.ops_map[key] = [op]
                elif isinstance(op, (list, tuple)):
                    for o in op:
                        o._dtype = dtype
                    self.ops_map[key] = list(op)


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
