from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping, Sequence, TYPE_CHECKING
from radiate.wrapper import RsObject
from radiate.radiate import PyOp, _create_op

if TYPE_CHECKING:
    from radiate._typing import NodeValues


@dataclass(frozen=True, slots=True)
class OpsConfig:
    vertex: NodeValues | None = None
    edge: NodeValues | None = None
    output: NodeValues | None = None
    leaf: NodeValues | None = None
    root: NodeValues | None = None
    values: Mapping[str, Sequence[Op]] | None = None

    def build_ops_map(
        self, input_size: int, fill_invalid: bool = False
    ) -> dict[str, list[Op]]:
        def inner():
            base = {"input": [Op.var(i) for i in range(input_size)]} | {
                "leaf": [Op.var(i) for i in range(input_size)]
            }

            if self.values is not None:
                # if it's mapping, just merge
                merged = dict(self.values)
                result = merged | base
                {
                    node_type: [op.__backend__() for op in ops]
                    for node_type, ops in result.items()
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
                ops_map["vertex"] = [op.__backend__() for op in Op.default_vertex_ops()]
            if "edge" not in ops_map:
                ops_map["edge"] = [op.__backend__() for op in Op.default_edge_ops()]
            if "output" not in ops_map:
                ops_map["output"] = [Op.linear().__backend__()]

        return ops_map


class Op(RsObject[PyOp]):
    @classmethod
    def var(cls, idx: int = 0) -> Op:
        return cls.from_rust(_create_op("var", {"index": idx}))

    @classmethod
    def const(cls, value: float) -> Op:
        return cls.from_rust(_create_op("constant", {"value": value}))

    @classmethod
    def add(cls) -> Op:
        return cls.from_rust(_create_op("add"))

    @classmethod
    def sub(cls) -> Op:
        return cls.from_rust(_create_op("sub"))

    @classmethod
    def mul(cls) -> Op:
        return cls.from_rust(_create_op("mul"))

    @classmethod
    def div(cls) -> Op:
        return cls.from_rust(_create_op("div"))

    @classmethod
    def sigmoid(cls) -> Op:
        return cls.from_rust(_create_op("sigmoid"))

    @classmethod
    def weight(cls) -> Op:
        return cls.from_rust(_create_op("weight"))

    @classmethod
    def relu(cls) -> Op:
        return cls.from_rust(_create_op("relu"))

    @classmethod
    def tanh(cls) -> Op:
        return cls.from_rust(_create_op("tanh"))

    @classmethod
    def linear(cls) -> Op:
        return cls.from_rust(_create_op("linear"))

    @classmethod
    def sum(cls) -> Op:
        return cls.from_rust(_create_op("sum"))

    @classmethod
    def prod(cls) -> Op:
        return cls.from_rust(_create_op("prod"))

    @classmethod
    def diff(cls) -> Op:
        return cls.from_rust(_create_op("diff"))

    @classmethod
    def pow(cls) -> Op:
        return cls.from_rust(_create_op("pow"))

    @classmethod
    def log(cls) -> Op:
        return cls.from_rust(_create_op("log"))

    @classmethod
    def sin(cls) -> Op:
        return cls.from_rust(_create_op("sin"))

    @classmethod
    def cos(cls) -> Op:
        return cls.from_rust(_create_op("cos"))

    @classmethod
    def identity(cls) -> Op:
        return cls.from_rust(_create_op("identity"))

    @classmethod
    def neg(cls) -> Op:
        return cls.from_rust(_create_op("neg"))

    @classmethod
    def sqrt(cls) -> Op:
        return cls.from_rust(_create_op("sqrt"))

    @classmethod
    def abs(cls) -> Op:
        return cls.from_rust(_create_op("abs"))

    @classmethod
    def exp(cls) -> Op:
        return cls.from_rust(_create_op("exp"))

    @classmethod
    def tan(cls) -> Op:
        return cls.from_rust(_create_op("tan"))

    @classmethod
    def ceil(cls) -> Op:
        return cls.from_rust(_create_op("ceil"))

    @classmethod
    def floor(cls) -> Op:
        return cls.from_rust(_create_op("floor"))

    @classmethod
    def max(cls) -> Op:
        return cls.from_rust(_create_op("max"))

    @classmethod
    def min(cls) -> Op:
        return cls.from_rust(_create_op("min"))

    @classmethod
    def leaky_relu(cls) -> Op:
        return cls.from_rust(_create_op("leaky_relu"))

    @classmethod
    def elu(cls) -> Op:
        return cls.from_rust(_create_op("elu"))

    @classmethod
    def mish(cls) -> Op:
        return cls.from_rust(_create_op("mish"))

    @classmethod
    def swish(cls) -> Op:
        return cls.from_rust(_create_op("swish"))

    @classmethod
    def softplus(cls) -> Op:
        return cls.from_rust(_create_op("softplus"))

    @staticmethod
    def default_vertex_ops() -> list[Op]:
        return [
            Op.add(),
            Op.sub(),
            Op.mul(),
            Op.div(),
            Op.sin(),
            Op.cos(),
            Op.tanh(),
            Op.relu(),
            Op.linear(),
        ]

    @staticmethod
    def default_edge_ops() -> list[Op]:
        return [Op.weight()]

    @staticmethod
    def all_ops() -> list[Op]:
        return [
            Op.add(),
            Op.sub(),
            Op.mul(),
            Op.div(),
            Op.sum(),
            Op.prod(),
            Op.diff(),
            Op.neg(),
            Op.pow(),
            Op.sqrt(),
            Op.abs(),
            Op.exp(),
            Op.log(),
            Op.sin(),
            Op.cos(),
            Op.tan(),
            Op.ceil(),
            Op.floor(),
            Op.max(),
            Op.min(),
            Op.sigmoid(),
            Op.tanh(),
            Op.relu(),
            Op.leaky_relu(),
            Op.elu(),
            Op.linear(),
            Op.mish(),
            Op.swish(),
            Op.identity(),
            Op.weight(),
        ]
