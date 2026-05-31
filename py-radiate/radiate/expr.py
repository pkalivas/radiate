from __future__ import annotations
from typing import Any

from radiate._bridge.input import RsObject
from radiate._typing import RdDataType
from radiate.radiate import PyExpr


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
      :meth:`generation`, :meth:`pi_signal`, and friends.
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
    def error_from(cls, metric: str, target: float) -> Expr:
        """
        Relative error from a target: ``(metric - target) / target``.

        Reads ``metric.last_value`` and computes a signed normalized error.
        Fuses to a single Affine node.

        Parameters
        ----------
        metric : str
            Name of the metric to read.
        target : float
            Reference value. Must be non-zero.
        """
        return cls.from_rust(PyExpr.error_from(metric, target))

    @classmethod
    def is_converged(cls, metric: str, window: int, epsilon: float) -> Expr:
        """
        Bool. True when ``|first - last|`` over a rolling window of size
        ``window`` falls below ``epsilon``.

        Useful as an early-stop limit:

        >>> import radiate as rd
        >>> rd.Limit.expr(rd.Expr.is_converged("scores.best", 50, 1e-4))
        """
        return cls.from_rust(PyExpr.is_converged(metric, window, epsilon))

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

    @classmethod
    def pi_signal(cls, metric: str, target: float, gain: float, window: int) -> Expr:
        """
        Adaptive correction signal centered at 1.0.

        Evaluates each generation to::

            signal = 1 + gain * (rolling_mean(metric, window) - target) / target

        Interpretation
        --------------
        - metric ≈ target  →  signal ≈ 1.0  (no correction)
        - metric > target  →  signal > 1.0  (push controlled variable up)
        - metric < target  →  signal < 1.0  (push down)

        Use as a multiplier on the variable you want to control:

        >>> import radiate as rd
        >>> anchor = rd.Expr.select("species.distance").rolling(10).max()
        >>> signal = rd.Expr.pi_signal("count.species", target=8, gain=0.5, window=10)
        >>> threshold = (anchor * signal).clamp(0.005, 2.0)

        Parameters
        ----------
        metric : str
            Metric to track. Read as a rolling mean over the last ``window`` gens.
        target : float
            Desired equilibrium value of the metric. Must be non-zero.
        gain : float
            Correction aggressiveness. 0 = no response, 0.5 is a typical starting
            point, 1.0+ may oscillate.
        window : int
            Rolling-mean smoothing window.

        Notes
        -----
        Structurally a proportional controller with a smoothed input. Loosely
        called "PI" because the rolling mean approximates an integral over the
        window, but no accumulating integral state is kept. For a true
        target-seeking controller, multiply ``pi_signal`` by the controller's
        own previous output (e.g. ``rd.Expr.select("species.threshold").last()``)
        rather than a static anchor.
        """
        return cls.from_rust(PyExpr.pi_signal(metric, target, gain, window))

    @classmethod
    def p50(cls, metric: str) -> Expr:
        """
        Streaming median (P²) of a metric. Constant memory; sees every observation
        since the expression was constructed. For an exact windowed median, use
        ``rd.Expr.select(metric).rolling(N).quantile(0.5)``.
        """
        return cls.from_rust(PyExpr.p50(metric))

    @classmethod
    def p95(cls, metric: str) -> Expr:
        """Streaming 95th-percentile (P²) of a metric."""
        return cls.from_rust(PyExpr.p95(metric))

    @classmethod
    def p99(cls, metric: str) -> Expr:
        """Streaming 99th-percentile (P²) of a metric."""
        return cls.from_rust(PyExpr.p99(metric))

    @classmethod
    def quantile_stream(cls, metric: str, q: float) -> Expr:
        """
        Streaming P² quantile estimator for an arbitrary ``q`` in ``(0, 1)``.

        Constant memory, constant per-update time. Approximate but accurate for
        unimodal distributions. For exact quantiles over a recent window, use
        ``rd.Expr.select(metric).rolling(N).quantile(q)``.
        """
        return cls.select(metric).streaming_quantile(q)

    @classmethod
    def track(
        cls,
        metric: str,
        target: float,
        *,
        anchor: Expr | float = 1.0,
        gain: float = 0.5,
        window: int = 10,
        lo: float = 0.0001,
        hi: float = 1.0,
    ) -> Expr:
        """
        High-level adaptive rate that nudges a controlled variable based on how
        far a metric sits from a target. Equivalent to::

            clamp(
                anchor * (1 + gain * (rolling_mean(metric, window) - target) / target),
                lo,
                hi,
            )

        i.e. ``clamp(anchor * pi_signal(metric, target, gain, window), lo, hi)``.

        Use this for a rate (mutation rate, crossover rate, speciation threshold,
        etc.) that auto-corrects to hold an observed metric near a target value.

        Choosing the anchor — proportional vs. target-seeking
        -----------------------------------------------------
        The ``anchor`` is the reference the correction multiplies, and it
        determines what kind of controller you get:

        * **Static anchor (a scalar, or a metric unrelated to the output).**
          Each generation the rate is recomputed from scratch as
          ``anchor * (1 + gain*error)``. This is a *proportional* controller: it
          pushes in the right direction but settles at a non-zero steady-state
          error (it parks *near* the target, not *on* it), since the only way the
          error reaches zero is if ``anchor`` already happens to equal the
          output that hits the target.
        * **Dynamic anchor = the controller's own previous output.** Passing
          ``anchor = rd.Expr.select("<this rate's metric>").last()`` turns the
          recurrence into ``out_t = out_{t-1} * (1 + gain*error)``, a
          multiplicative *integrator*. At steady state ``out_t == out_{t-1}``
          forces ``error == 0``, so it genuinely seeks the target with no offset.
          This requires the engine to record the rate's output as a metric the
          next generation can read (the speciation threshold is recorded as
          ``species.threshold``).

          **With this self-referential form, ``lo`` must be strictly positive.**
          Zero is an absorbing state: on the first generation the metric is not
          yet recorded, so the anchor is ``Null`` and clamps to ``lo``; if that
          seed is ``0`` then every later step is ``0 * signal == 0`` and the
          controller is stuck at zero forever. ``lo`` doubles as both the
          minimum output and the generation-zero seed, so keep it small but
          non-zero (the default ``0.0001`` is safe).

        Parameters
        ----------
        metric : str
            Metric to observe. Read as a rolling mean over ``window`` generations
            (not the raw last value), so the controller reacts to a smoothed
            signal rather than single-generation noise.
        target : float
            Desired equilibrium value of the metric. Must be non-zero.
        anchor : Expr or float, default 1.0
            Reference the correction multiplies. A scalar (or unrelated Expr)
            gives a proportional controller; the rate's own previous output gives
            a target-seeking integrator. See the discussion above.
        gain : float, default 0.5
            Correction aggressiveness. 0 = no response; ~0.1–0.5 is a sane band;
            1.0+ may oscillate, especially with an integrating anchor.
        window : int, default 10
            Rolling-mean smoothing window for the metric. Larger = smoother but
            adds loop lag (more oscillation risk with an integrating anchor).
        lo, hi : float, default 0.0001 and 1.0
            Clamp bounds on the final rate. ``lo`` must be ``> 0`` when the
            anchor is the controller's own previous output (see above) — it is
            both the floor and the generation-zero seed, and a zero seed traps
            the integrator at zero.

        Examples
        --------
        Proportional — rate centered at 0.1, corrected by the score error::

            rate = rd.Expr.track(
                "scores.best",
                target=0.0,
                anchor=0.1,
                gain=0.5,
                window=10,
                lo=0.01,
                hi=0.3,
            )

        Target-seeking — hold the species count near 8 by driving the speciation
        threshold, anchored on its own previous value so it converges with no
        steady-state offset::

            threshold = rd.Expr.track(
                "count.species",
                target=8.0,
                anchor=rd.Expr.select("species.threshold").last(),
                gain=0.3,
                window=8,
                lo=0.005,
                hi=2.0,
            )
        """
        signal = cls.pi_signal(metric, target, gain, window)
        if isinstance(anchor, Expr):
            product = anchor * signal
        else:
            product = signal * float(anchor)
        return product.clamp(lo, hi)

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

    def apply(self, value: Any) -> Any:
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
        return self.__backend__().evaluate(value)

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

    def affine(self, scale: float, bias: float) -> Expr:
        """
        scale * self + bias. Fuses into a single Affine node.

        Consecutive .affine() calls collapse algebraically:
            x.affine(2, 3).affine(4, 5) → Affine(scale=8, bias=17)

        Equivalent to (self * scale + bias) but stored as one IR node.
        """
        return Expr.from_rust(self.__backend__().affine(scale, bias))

    def streaming_quantile(self, q: float) -> Expr:
        """
        Streaming P² quantile over THIS expression's evaluated stream.
        Constant memory (five markers), constant per-eval cost.

        Composes with any child — quantile of a smoothed mean, a literal,
        a binary op, etc.:

            # streaming p95 of a 20-gen rolling mean
            rd.Expr.select("scores.best").rolling(20).mean().streaming_quantile(0.95)

        For an exact quantile over a recent window, use `.rolling(N).quantile(q)`.

        Parameters
        ----------
        q : float
            Quantile to track, in the open interval (0, 1).
        """
        return Expr.from_rust(self.__backend__().streaming_quantile(q))
