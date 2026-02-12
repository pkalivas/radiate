from __future__ import annotations

from typing import Any, Callable

from radiate.codec import (
    FloatCodec,
    IntCodec,
    CharCodec,
    BitCodec,
    GraphCodec,
    TreeCodec,
    PermutationCodec,
)

from radiate.operators import SelectorBase, AlterBase, DistanceBase, Executor, LimitBase
from radiate.fitness import FitnessBase, Regression
from radiate.genome import Population, GeneType, Gene
from radiate.gp import Graph, Tree, Op
from radiate.dtype import Float64, Int64

from radiate._bridge.input import EngineInput, EngineInputType
from radiate._typing import (
    AtLeastOne,
    Subscriber,
    RdDataType,
    Encoding,
    Decoding,
)

from .builder import EngineBuilder
from .generation import Generation
from .option import EngineCheckpoint, EngineLog, EngineUi


class Engine[G: Gene, T]:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: Encoding[T] | None = None,
        **kwargs: Any,
    ):
        self._engine = None
        if codec is not None:
            self._builder = EngineBuilder._default(
                codec.gene_type, codec=codec, **kwargs
            )
        else:
            self._builder = EngineBuilder._default(GeneType.Float, **kwargs)

    @classmethod
    def float(
        cls,
        shape: AtLeastOne[int] = 1,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = Float64,
        use_numpy: bool = False,
    ) -> Engine[Gene[float], Decoding[float]]:
        """Create a genetic engine for optimizing floating-point values."""
        instance = cls.__new__(cls)
        codec = FloatCodec(
            shape,
            init_range=init_range,
            bounds=bounds,
            dtype=dtype,
            use_numpy=use_numpy,
        )

        instance._engine = None
        instance._builder = EngineBuilder._default(GeneType.FLOAT).set_codec(codec)
        return instance

    @classmethod
    def int(
        cls,
        shape: AtLeastOne[int] = 1,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = Int64,
        use_numpy: bool = False,
    ) -> Engine[Gene[int], Decoding[int]]:
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
        instance._engine = None
        return instance

    @classmethod
    def char(
        cls,
        shape: AtLeastOne[int] = 1,
        char_set: set[str] | None = None,
    ) -> Engine[Gene[str], Decoding[str]]:
        """Create a genetic engine for optimizing character values."""
        instance = cls.__new__(cls)
        codec = CharCodec(shape, char_set=char_set)

        instance._builder = EngineBuilder._default(GeneType.CHAR).set_codec(codec)
        instance._engine = None
        return instance

    @classmethod
    def bit(
        cls, shape: int | tuple[int, ...] | list[int] = 1, use_numpy: bool = False
    ) -> Engine[Gene[bool], Decoding[bool]]:
        """Create a genetic engine for optimizing boolean values."""
        instance = cls.__new__(cls)
        codec = BitCodec(shape, use_numpy=use_numpy)

        instance._builder = EngineBuilder._default(GeneType.BIT).set_codec(codec)
        instance._engine = None
        return instance

    @classmethod
    def permutation(cls, items: list[T]) -> Engine[Gene[T], list[T]]:
        """Create a genetic engine for optimizing permutations of a list of items."""
        instance = cls.__new__(cls)
        codec = PermutationCodec(items)

        instance._builder = EngineBuilder._default(GeneType.PERMUTATION).set_codec(
            codec
        )
        instance._engine = None
        return instance

    @classmethod
    def graph(
        cls,
        shape: tuple[int, int],
        vertex: Op | list[Op] | None = None,
        edge: Op | list[Op] | None = None,
        output: Op | list[Op] | None = None,
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
        instance._engine = None
        return instance

    @classmethod
    def tree(
        cls,
        shape: tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Op | list[Op] | None = None,
        leaf: Op | list[Op] | None = None,
        root: Op | list[Op] | None = None,
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
        instance._engine = None
        return instance

    def __iter__(self):
        """Allow unpacking the engine into its components."""
        while True:
            yield self.__next__()

    def __next__(self) -> Generation[T]:
        """Get the next generation from the engine."""
        if self._engine is None:
            self._engine = self._builder.build()

        try:
            generation = self._engine.next()
            return Generation.from_rust(generation)
        except StopIteration:
            self._engine = None
            raise

    def run(
        self,
        limits: LimitBase | list[LimitBase] = [],
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
        batch: bool = False,
    ) -> Engine[G, T]:
        """
        Configure regression fitness.

        Accepts:
        - (features, targets)
        - a DataFrame (polars / pandas)
        """
        self._builder.set_fitness(
            Regression(
                features,
                targets,
                target=target,
                feature_cols=feature_cols,
                loss=loss,
                batch=batch,
            )
        )
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
