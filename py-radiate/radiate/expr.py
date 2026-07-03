from __future__ import annotations

from typing import TYPE_CHECKING, Any

from radiate.radiate import PyExpr

from ._bridge.input import RsObject
from ._typing import RdDataType

if TYPE_CHECKING:
    from .engine.metrics import MetricSet


def _coerce(value, *, allow_str: bool = False):
    if isinstance(value, Expr):
        return value.__backend__()
    types = (float, int, str) if allow_str else (float, int)
    if isinstance(value, types):
        return PyExpr.literal(value)
    raise TypeError(f"Expected Expr or scalar, got {type(value).__name__}")


def _binary_op(backend_name: str, *, allow_str: bool = False):
    def op(self, rhs):
        return Expr.from_rust(
            getattr(self.__backend__(), backend_name)(_coerce(rhs, allow_str=allow_str))
        )

    return op


class Then(RsObject):
    def __init__(self, condition: Expr | int, then_expr: Expr):
        self.condition = condition
        self.then_expr = then_expr

    def __repr__(self) -> str:
        return f"Then(condition={self.condition}, then={self.then_expr})"

    def otherwise(self, else_expr: Expr | float | int | str) -> Expr:
        else_backend = _coerce(else_expr, allow_str=True)
        if isinstance(self.condition, int):
            return Expr.from_rust(
                PyExpr.every(
                    self.condition,
                    self.then_expr.__backend__(),
                    else_backend,
                )
            )
        return Expr.from_rust(
            PyExpr.when_then_otherwise(
                condition=self.condition.__backend__(),
                then_expr=self.then_expr.__backend__(),
                else_expr=else_backend,
            )
        )


class When(RsObject):
    def __init__(self, condition: Expr):
        self.condition = condition

    def __repr__(self) -> str:
        return f"When(condition={self.condition})"

    def then(self, then_expr: Expr | float | int | str) -> Then:
        return Then(
            condition=self.condition,
            then_expr=Expr.from_rust(_coerce(then_expr, allow_str=True)),
        )


class Every(RsObject):
    def __init__(self, interval: int):
        self.interval = interval

    def __repr__(self) -> str:
        return f"Every(interval={self.interval})"

    def then(self, then_expr: Expr | float | int | str) -> Then:
        return Then(
            condition=self.interval,
            then_expr=Expr.from_rust(_coerce(then_expr, allow_str=True)),
        )


class Expr(RsObject):
    """
    A node in the Radiate metric-expression DSL.

    An ``Expr`` is a small, composable formula evaluated against the engine's
    live ``MetricSet`` once per generation. Use it to drive adaptive rates
    (``rd.Rate.expr`` / passing an ``Expr`` to ``.diversity(...)``), stopping
    conditions (``rd.Limit.expr``), or derived metrics (``.metrics(...)``).

    The class is both the value type and the namespace for building expressions:

    * **Constructors** are classmethods that start a new expression —
      :meth:`select`, :meth:`lit`, :meth:`when`, :meth:`every`,
      :meth:`generation`, :meth:`stagnation`, and friends.
    * **Combinators** are instance methods that transform an existing expression
      — :meth:`rolling`, :meth:`mean`, :meth:`clamp`, the arithmetic
      (``+ - * / **``) and comparison (``< <= > >= == !=``) operators, etc.

    Examples
    --------
    >>> import radiate as rd
    >>> # smoothed relative error of the species count vs a target of 8
    >>> rd.Expr.select("count.species").rolling(10).mean().error(8.0)
    """

    def __repr__(self) -> str:
        return f"Expr({self.__backend__().__repr__()})"

    def __str__(self) -> str:
        return self.__backend__().__str__()

    # ── construction (classmethods) ─────────────────────────────────────────

    @classmethod
    def select(cls, metric: str) -> Expr:
        """
        Read a metric by name. Defaults to the metric's last recorded value;
        chain an aggregation (:meth:`mean`, :meth:`max`, ...) to read a stat.

        >>> import radiate as rd
        >>> rd.Expr.select("scores.best")
        """
        return cls.from_rust(PyExpr.select(metric))

    @classmethod
    def lit(cls, value: float | int | str) -> Expr:
        """
        A literal constant.

        >>> import radiate as rd
        >>> rd.Expr.lit(0.01)
        """
        return cls.from_rust(PyExpr.literal(value))

    @classmethod
    def when(cls, condition: Expr) -> When:
        """
        Begin a conditional. Pair with ``.then(...).otherwise(...)``.

        >>> import radiate as rd
        >>> (
        ...     rd.Expr.when(rd.Expr.select("scores.best") < 0.01)
        ...     .then(rd.Expr.select("scores.best").mean())
        ...     .otherwise(rd.Expr.lit(1.0))
        ... )
        """
        return When(condition=condition)

    @classmethod
    def every(cls, interval: int) -> Every:
        """
        A schedule that is "active" every ``interval`` generations. Pair with
        ``.then(...).otherwise(...)`` to switch between two expressions.

        >>> import radiate as rd
        >>> (
        ...     rd.Expr.every(10)
        ...     .then(rd.Expr.select("scores.best").rolling(10).stddev())
        ...     .otherwise(rd.Expr.select("scores.best"))
        ... )
        """
        return Every(interval=interval)

    @classmethod
    def element(cls) -> Expr:
        """The current element being evaluated (for element-wise contexts)."""
        return cls.from_rust(PyExpr.element())

    @classmethod
    def generation(cls) -> Expr:
        """
        The current generation index.

        >>> import radiate as rd
        >>> rd.Expr.generation()
        """
        return cls.from_rust(PyExpr.select("index"))

    @classmethod
    def stagnation(cls, metric: str, epsilon: float = 1e-4) -> Expr:
        """
        Counter: number of consecutive generations during which
        ``metric.last_value`` has stayed within ``epsilon`` of the last value
        considered an improvement. Resets to 0 on any change > ``epsilon``.

        Returns a Float32 counter, suitable for comparison via ``.gt()`` / ``.gte()``.
        """
        return cls.from_rust(PyExpr.stagnation(metric, epsilon))

    @classmethod
    def is_stagnant(cls, metric: str, patience: int, epsilon: float = 1e-4) -> Expr:
        """
        Bool. True when ``stagnation(metric, epsilon) >= patience``.

        Common pattern: gate a mutation-rate boost on this condition.

        >>> import radiate as rd
        >>> mut_rate = (
        ...     rd.Expr.when(rd.Expr.is_stagnant("scores.best", patience=20))
        ...     .then(rd.Expr.lit(0.30))
        ...     .otherwise(rd.Expr.lit(0.05))
        ... )
        """
        return cls.from_rust(PyExpr.is_stagnant(metric, patience, epsilon))

    # ── operator overloads ──────────────────────────────────────────────────

    def __lt__(self, other):
        return self.lt(other)

    def __le__(self, other):
        return self.lte(other)

    def __gt__(self, other):
        return self.gt(other)

    def __ge__(self, other):
        return self.gte(other)

    def __eq__(self, other):
        return self.eq(other)

    def __ne__(self, other):
        return self.ne(other)

    def __and__(self, other):
        return self.and_(other)

    def __or__(self, other):
        return self.or_(other)

    def __invert__(self):
        return self.not_()

    def __neg__(self):
        return self.neg_()

    def __abs__(self):
        return self.abs_()

    def __add__(self, other):
        return self.add(other)

    def __sub__(self, other):
        return self.sub(other)

    def __mul__(self, other):
        return self.mul(other)

    def __pow__(self, other):
        return self.pow(other)

    def __truediv__(self, other):
        return self.div(other)

    # ── combinators (instance methods) ───────────────────────────────────────

    def eval(self, value: MetricSet) -> Any:
        """
        Apply the expression to a given value. This is useful for evaluating the expression with specific inputs.

        Parameters
        ----------
        value
            The value to apply the expression to.

        Returns
        -------
        Any
            The result of applying the expression to the input value.
        """
        return self.__backend__().evaluate(value.__backend__())

    def time(self) -> Expr:
        return Expr.from_rust(self.__backend__().time())

    def rolling(self, window: int) -> Expr:
        return Expr.from_rust(self.__backend__().rolling(window))

    def first(self) -> Expr:
        return Expr.from_rust(self.__backend__().first())

    def last(self) -> Expr:
        return Expr.from_rust(self.__backend__().last())

    def sum(self) -> Expr:
        return Expr.from_rust(self.__backend__().sum())

    def mean(self) -> Expr:
        return Expr.from_rust(self.__backend__().mean())

    def stddev(self) -> Expr:
        return Expr.from_rust(self.__backend__().stddev())

    def min(self) -> Expr:
        return Expr.from_rust(self.__backend__().min())

    def max(self) -> Expr:
        return Expr.from_rust(self.__backend__().max())

    def var(self) -> Expr:
        return Expr.from_rust(self.__backend__().var())

    def skew(self) -> Expr:
        return Expr.from_rust(self.__backend__().skew())

    def count(self) -> Expr:
        return Expr.from_rust(self.__backend__().count())

    def unique(self) -> Expr:
        return Expr.from_rust(self.__backend__().unique())

    def debug(self) -> Expr:
        return Expr.from_rust(self.__backend__().debug())

    def slope(self) -> Expr:
        return Expr.from_rust(self.__backend__().slope())

    lt = _binary_op("lt")
    lte = _binary_op("lte")
    gt = _binary_op("gt")
    gte = _binary_op("gte")
    eq = _binary_op("eq", allow_str=True)
    ne = _binary_op("ne", allow_str=True)

    def and_(self, rhs: Expr) -> Expr:
        return Expr.from_rust(self.__backend__().and_(rhs.__backend__()))

    def or_(self, rhs: Expr) -> Expr:
        return Expr.from_rust(self.__backend__().or_(rhs.__backend__()))

    def not_(self) -> Expr:
        return Expr.from_rust(self.__backend__().not_())

    def neg_(self) -> Expr:
        return Expr.from_rust(self.__backend__().neg_())

    def abs_(self) -> Expr:
        return Expr.from_rust(self.__backend__().abs_())

    add = _binary_op("add_")
    sub = _binary_op("sub_")
    mul = _binary_op("mul_")
    div = _binary_op("div_")
    pow = _binary_op("pow")

    def clamp(self, min: Expr | float | int, max: Expr | float | int) -> Expr:
        return Expr.from_rust(self.__backend__().clamp_(_coerce(min), _coerce(max)))

    def cast(self, to: RdDataType) -> Expr:
        return Expr.from_rust(self.__backend__().cast(str(to)))

    def error(self, target: float | int) -> Expr:
        """
        Relative error from a target: (self - target) / target.

        Fuses into a single Affine node. Useful after a rolling mean
        or other transform to compute signed error:

            rd.Expr.select("count.species").rolling(10).mean().error(8.0)
        """
        return Expr.from_rust(self.__backend__().error(target))

    def quantile(self, q: float) -> Expr:
        """
        Exact quantile over the rolling buffer this expression already has.
        Requires a `.rolling(N)` upstream — otherwise treats the single
        value as a 1-element distribution.
        """
        return Expr.from_rust(self.__backend__().quantile(q))
