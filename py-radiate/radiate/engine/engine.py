from __future__ import annotations

from typing import Any, Callable

from .._typing import (
    Subscriber,
    RdDataType,
    IntDecoding,
    FloatDecoding,
    BoolDecoding,
    StringDecoding,
    NodeValues,
)

from .builder import EngineBuilder, EngineConfig
from .generation import Generation
from ..codec import (
    CodecBase,
    FloatCodec,
    IntCodec,
    CharCodec,
    BitCodec,
    GraphCodec,
    TreeCodec,
    PermutationCodec,
)

from ..operators.input import EngineInput, EngineInputType
from ..operators.selector import SelectorBase
from ..operators.alterer import AlterBase
from ..operators.distance import DistanceBase
from ..operators.executor import Executor
from ..fitness import FitnessBase, Regression
from ..operators.limit import LimitBase
from .option import EngineCheckpoint, EngineLog, EngineUi
from ..gp import Graph, Tree, Op
from ..genome.population import Population
from ..genome import GeneType
from ..genome.gene import Gene

from ..dtype import Float64, Int64


class Engine[G, T]:
    """
    Base class for genetic engines. This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    _builder: EngineBuilder = None

    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: CodecBase[G, T],
        fitness_func: Callable[[T], Any] | FitnessBase[T],
        **kwargs: Any,
    ):
        cfg = EngineConfig(codec=codec, fitness_func=fitness_func, **kwargs)
        self._builder = EngineBuilder.from_config(cfg)

    @classmethod
    def float(
        cls,
        shape: int | tuple[int, ...] | list[int] = 1,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = Float64,
        use_numpy: bool = False,
    ) -> Engine[Gene[float], FloatDecoding]:
        """Create a genetic engine for optimizing floating-point values."""
        instance = cls.__new__(cls)
        codec = FloatCodec(
            shape,
            init_range=init_range,
            bounds=bounds,
            dtype=dtype,
            use_numpy=use_numpy,
        )

        instance._builder = EngineBuilder._default(GeneType.FLOAT).set_codec(codec)
        return instance

    @classmethod
    def int(
        cls,
        shape: int | tuple[int, ...] | list[int] = 1,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = Int64,
        use_numpy: bool = False,
    ) -> Engine[Gene[int], IntDecoding]:
        """Create a genetic engine for optimizing integer values."""
        instance = cls.__new__(cls)
        codec = IntCodec(
            shape,
            init_range=init_range,
            bounds=bounds,
            dtype=dtype,
            use_numpy=use_numpy,
        )

        instance._builder = EngineBuilder._default(GeneType.INT).set_codec(codec)
        return instance

    @classmethod
    def char(
        cls,
        shape: int | tuple[int, ...] | list[int] = 1,
        char_set: set[str] | None = None,
    ) -> Engine[Gene[str], StringDecoding]:
        """Create a genetic engine for optimizing character values."""
        instance = cls.__new__(cls)
        codec = CharCodec(shape, char_set=char_set)

        instance._builder = EngineBuilder._default(GeneType.CHAR).set_codec(codec)
        return instance

    @classmethod
    def bit(
        cls, shape: int | tuple[int, ...] | list[int] = 1, use_numpy: bool = False
    ) -> Engine[Gene[bool], BoolDecoding]:
        """Create a genetic engine for optimizing boolean values."""
        instance = cls.__new__(cls)
        codec = BitCodec(shape, use_numpy=use_numpy)

        instance._builder = EngineBuilder._default(GeneType.BIT).set_codec(codec)
        return instance

    @classmethod
    def permutation(cls, items: list[T]) -> Engine[Gene[T], list[T]]:
        """Create a genetic engine for optimizing permutations of a list of items."""
        instance = cls.__new__(cls)
        codec = PermutationCodec(items)

        instance._builder = EngineBuilder._default(GeneType.PERMUTATION).set_codec(
            codec
        )
        return instance

    @classmethod
    def graph(
        cls,
        shape: tuple[int, int],
        vertex: NodeValues | None = None,
        edge: NodeValues | None = None,
        output: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
        max_nodes: int | None = None,
    ) -> Engine[Gene[Op], Graph]:
        """Create a genetic engine for optimizing graph structures."""
        instance = cls.__new__(cls)
        codec = GraphCodec(
            graph_type="directed",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

        instance._builder = EngineBuilder._default(GeneType.GRAPH).set_codec(codec)
        return instance

    @classmethod
    def tree(
        cls,
        shape: tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: NodeValues | None = None,
        leaf: NodeValues | None = None,
        root: NodeValues | None = None,
        values: dict[str, list[Op]] | list[tuple[str, list[Op]]] | None = None,
    ) -> Engine[Gene[Op], Tree]:
        """Create a genetic engine for optimizing tree structures."""
        instance = cls.__new__(cls)
        codec = TreeCodec(
            shape,
            min_depth=min_depth,
            max_size=max_size,
            vertex=vertex,
            leaf=leaf,
            root=root,
            values=values,
        )

        instance._builder = EngineBuilder._default(GeneType.TREE).set_codec(codec)
        return instance

    def run(
        self,
        limits: LimitBase | list[LimitBase],
        log: bool | EngineLog = False,
        checkpoint: tuple[int, str] | EngineCheckpoint | None = None,
        ui: bool | EngineUi = False,
    ) -> Generation[T]:
        """Run the engine with the given limits.
        Args:
            limits: A single Limit or a list of Limits to apply to the engine.
            log: If True, enables logging for the generation process.
            checkpoint: If provided, enables checkpointing at the specified interval and path. Checkpoint can be
                        specified as a tuple (interval, path) or an EngineCheckpoint instance.
            ui: If True, enables a user interface for monitoring the evolution process.
        Returns:
            Generation: The resulting generation after running the engine.
        Raises:
            ValueError: If limits are not provided or invalid, or if other parameters are invalid.

        Example:
        ---------
        >>> engine.run(rd.ScoreLimit(0.0001), log=True)
        """

        if limits is not None:
            if isinstance(limits, LimitBase):
                limits = [limits]
            elif isinstance(limits, list):
                if len(limits) == 0:
                    raise ValueError(
                        "At least one limit must be provided to run the engine."
                    )
            else:
                raise TypeError(
                    "Limits must be a LimitBase or a list of LimitBase instances."
                )
        else:
            raise ValueError("At least one limit must be provided to run the engine.")

        engine = self._builder.build()

        limit_inputs = [
            EngineInput(
                input_type=EngineInputType.Limit,
                component=lim.component,
                allowed_genes=GeneType.all(),
                **lim.args,
            ).__backend__()
            for lim in limits
        ]

        # configure the logging option
        log_option = log if isinstance(log, EngineLog) else EngineLog(enable=log)

        # configure the checkpoint option
        checkpoint_option = (
            checkpoint if isinstance(checkpoint, EngineCheckpoint) else None
        )
        if checkpoint_option is None and isinstance(checkpoint, tuple):
            checkpoint_option = EngineCheckpoint(
                interval=checkpoint[0], path=checkpoint[1]
            )

        # configure the UI option
        ui_option = ui if isinstance(ui, EngineUi) else None
        if ui_option is None and ui is True:
            ui_option = EngineUi()

        options = list(
            map(
                lambda opt: opt.__backend__(),
                filter(
                    lambda opt: opt is not None,
                    [log_option, checkpoint_option, ui_option],
                ),
            )
        )

        return Generation.from_rust(engine.run(limit_inputs, options))

    def fitness(
        self, fitness_func: Callable[[T], Any] | FitnessBase[T]
    ) -> Engine[G, T]:
        """Set the fitness function for the engine."""
        self._builder.set_fitness(fitness_func)
        return self

    def regression(
        self,
        features: Any,
        targets: Any | None = None,
        *,
        target: str | None = None,
        feature_cols: list[str] | None = None,
        loss: str = "mse",
    ) -> Engine[G, T]:
        """
        Configure regression fitness.

        Accepts:
        - (features, targets)
        - a DataFrame (polars / pandas)
        """
        from ..utils._normalize import _normalize_regression_data

        X, y = _normalize_regression_data(
            features,
            targets,
            feature_cols=feature_cols,
            target_col=target,
        )

        self._builder.set_fitness(Regression(X, y, loss))
        self._builder.set_objective("min")
        return self

    def select(
        self,
        offspring: SelectorBase | None = None,
        survivor: SelectorBase | None = None,
        frac: float | None = None,
    ) -> Engine[G, T]:

        if offspring is not None:
            self._builder.set_offspring_selector(offspring)
        if survivor is not None:
            self._builder.set_survivor_selector(survivor)

        if frac is not None:
            if not (0 < frac <= 1):
                raise ValueError("Offspring frac must be between 0 and 1.")
            self._builder.set_offspring_fraction(frac)

        return self

    def alters(self, *alters: AlterBase) -> Engine[G, T]:
        """Set the alters for the engine."""
        self._builder.set_alters(alters)
        return self

    def diversity(
        self, diversity: DistanceBase, species_threshold: float = 1.5
    ) -> Engine[G, T]:
        """Set the diversity strategy for the engine."""
        self._builder.set_diversity(diversity, species_threshold)
        return self

    def limit(self, *limits: LimitBase) -> Engine[G, T]:
        """Set the limits for the engine."""
        self._builder.set_limits(limits)
        return self

    def size(self, size: int) -> Engine[G, T]:
        """Set the population size.
        Args:
            size (int): The size of the population.
        Raises:
            ValueError: If size is less than or equal to 0.

        Example:
        ---------
        >>> engine.size(200)
        """
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")
        self._builder.set_population_size(size)
        return self

    def population(self, population: Population[G]) -> Engine[G, T]:
        """Set the initial population for the engine."""
        self._builder.set_population(population)
        return self

    def minimizing(self) -> Engine[G, T]:
        """Set the objectives to minimize.

        Example:
        ---------
        >>> engine.minimizing()
        """
        self._builder.set_objective("min")
        return self

    def maximizing(self) -> Engine[G, T]:
        """Set the objectives to maximize.

        Example:
        ---------
        >>> engine.maximizing()
        """
        self._builder.set_objective("max")
        return self

    def objective(
        self, *obj: str, front_range: tuple[int, int] | None = None
    ) -> Engine[G, T]:
        """Set the optimization objective(s) for the engine."""
        self._builder.set_objective(obj)
        if front_range is not None:
            self._builder.set_front_range(*front_range)
        return self

    def front_range(self, min: int, max: int) -> Engine[G, T]:
        """Set the range for the Pareto front in multi-objective optimization."""
        self._builder.set_front_range(min, max)
        return self

    def parallel(self, num_workers: int | None = None) -> Engine[G, T]:
        """Set the executor.
        Args:
            executor (Executor): The executor to use.
        Example:
        ---------
        >>> engine.executor(Executor.worker_pool())
        """
        executor = (
            Executor.WorkerPool()
            if num_workers is None
            else Executor.FixedSizedWorkerPool(num_workers)
        )

        self._builder.set_executor(executor)
        return self

    def subscribe(self, event_handler: Subscriber | None = None) -> Engine[G, T]:
        """Register an event handler.
        Args:
            event_handler: Union[
                Callable[[Any], None], list[Callable[[Any], None]], EventHandler, list[EventHandler]
            ] | None: The event handler(s) to register.
        Raises:
            TypeError: If event_handler is not callable or a list of callables.

        Example:
        ---------
        >>> engine.subscribe(my_event_handler)
        >>> engine.subscribe([handler1, handler2])
        """
        self._builder.set_subscribers(event_handler)
        return self

    def generation(self, generation: Generation[T] | None) -> Engine[G, T]:
        """Set the initial generation.
        Args:
            generation (Generation[T] | None): The initial generation to set.
        Example:
        ---------
        >>> engine.generation(initial_generation)
        """
        self._builder.set_generation(generation)
        return self
