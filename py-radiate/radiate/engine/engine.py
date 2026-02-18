from __future__ import annotations

from typing import Any, overload, Sequence
from collections.abc import Callable

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
from radiate.genome import Population, GeneType
from radiate.gp import Graph, Tree, Op
from radiate.dtype import Float64, Int64
from radiate.codec.base import CodecBase

from radiate._bridge.input import EngineInput, EngineInputType
from radiate._typing import (
    AtLeastOne,
    Subscriber,
    RdDataType,
    Encoding,
    ScalarDecoding,
    VectorDecoding,
    MatrixDecoding,
)

from .builder import EngineBuilder
from .generation import Generation
from .option import EngineCheckpoint, EngineLog, EngineUi


class Engine[G, T]:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: Encoding[G],
        **kwargs: Any,
    ):
        encoding = None
        if not isinstance(codec, CodecBase):
            # encoding = CodecBase.from_genes(codec)
            pass
        else:
            encoding = codec

        self._engine = None
        if encoding is not None:
            self._builder = EngineBuilder._default(
                encoding.gene_type, codec=encoding, **kwargs
            )

    # --- Float Engine Overloads ---
    @overload
    @staticmethod
    def float(
        *,
        shape: int = 1,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = Float64,
        use_numpy: bool = False,
    ) -> "Engine[float, ScalarDecoding[float]]": ...

    @overload
    @staticmethod
    def float(
        shape: int,
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = Float64,
        use_numpy: bool = False,
    ) -> "Engine[float, VectorDecoding[float]]": ...

    @overload
    @staticmethod
    def float(
        shape: Sequence[int],
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = Float64,
        use_numpy: bool = False,
    ) -> "Engine[float, MatrixDecoding[float]]": ...

    # --- End of Float Engine Overloads ---

    @staticmethod
    def float(
        shape: AtLeastOne[int] = 1,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = Float64,
        use_numpy: bool = False,
    ) -> "Engine[float, Any]":
        """Create a genetic engine for optimizing floating-point values."""
        return Engine(
            codec=FloatCodec(
                shape,
                init_range=init_range,
                bounds=bounds,
                dtype=dtype,
                use_numpy=use_numpy,
            )
        )

    # --- Integer Engine Overloads ---
    @overload
    @staticmethod
    def int(
        *,
        shape: int = 1,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = Int64,
        use_numpy: bool = False,
    ) -> "Engine[int, ScalarDecoding[int]]": ...

    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = Int64,
        use_numpy: bool = False,
    ) -> "Engine[int, VectorDecoding[int]]": ...

    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = Int64,
        use_numpy: bool = False,
    ) -> "Engine[int, MatrixDecoding[int]]": ...

    # --- End of Integer Engine Overloads ---

    @staticmethod
    def int(
        shape: AtLeastOne[int] = 1,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = Int64,
        use_numpy: bool = False,
    ) -> "Engine[int, Any]":
        """Create a genetic engine for optimizing integer values."""
        return Engine(
            codec=IntCodec(
                shape,
                init_range=init_range,
                bounds=bounds,
                dtype=dtype,
                use_numpy=use_numpy,
            )
        )

    # --- Character Engine Overloads ---
    @overload
    @staticmethod
    def char(
        shape: int,
        *,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, list[str]]": ...

    @overload
    @staticmethod
    def char(
        shape: Sequence[int],
        *,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, list[list[str]]]": ...

    # --- End of Character Engine Overloads ---

    @staticmethod
    def char(
        shape: AtLeastOne[int] = 1,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, Any]":
        """Create a genetic engine for optimizing character values."""
        return Engine(codec=CharCodec(shape, char_set=char_set))

    # --- Bit Engine Overloads ---
    @overload
    @staticmethod
    def bit(
        shape: int,
        use_numpy: bool = False,
    ) -> "Engine[bool, VectorDecoding[bool]]": ...

    @overload
    @staticmethod
    def bit(
        shape: Sequence[int],
        use_numpy: bool = False,
    ) -> "Engine[bool, MatrixDecoding[bool]]": ...

    # --- End of Bit Engine Overloads ---

    @staticmethod
    def bit(shape: AtLeastOne[int] = 1, use_numpy: bool = False) -> "Engine[bool, Any]":
        """Create a genetic engine for optimizing boolean values."""
        return Engine(codec=BitCodec(shape, use_numpy=use_numpy))

    @staticmethod
    def permutation(items: list[T]) -> Engine[T, list[T]]:
        """Create a genetic engine for optimizing permutations of a list of items."""
        return Engine(codec=PermutationCodec(items))

    @staticmethod
    def graph(
        shape: tuple[int, int],
        vertex: Op | list[Op] | None = None,
        edge: Op | list[Op] | None = None,
        output: Op | list[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
        graph_type: str = "directed",
    ) -> Engine[Op, Graph]:
        """Create a genetic engine for optimizing graph structures."""
        codec = GraphCodec(
            graph_type="directed",
            shape=shape,
            vertex=vertex,
            edge=edge,
            output=output,
            values=values,
            max_nodes=max_nodes,
        )

        return Engine(codec=codec)

    @staticmethod
    def tree(
        shape: tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Op | list[Op] | None = None,
        leaf: Op | list[Op] | None = None,
        root: Op | list[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
    ) -> Engine[Op, Tree]:
        """Create a genetic engine for optimizing tree structures."""
        return Engine(
            codec=TreeCodec(
                shape,
                min_depth=min_depth,
                max_size=max_size,
                vertex=vertex,
                leaf=leaf,
                root=root,
                values=values,
            )
        )

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
        *limits: LimitBase,
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
            [
                opt.__backend__()
                for opt in [log_option, checkpoint_option, ui_option]
                if opt is not None
            ]
        )

        return Generation.from_rust(engine.run(limit_inputs, options))

    def fitness(self, fitness_func: Callable[[Any], Any] | FitnessBase) -> Engine[G, T]:
        """
        Set the fitness function for the engine.

        Args:
            fitness_func: A callable that takes a decoded genome and returns a fitness score, or an instance of FitnessBase.

        Returns:
            Engine: The engine instance with the fitness function set.

        Example:
        ---------
        >>> def my_fitness_func(genome: list[float]) -> float:
        ...     return sum(genome)  # Example fitness function that sums the genome values
        ...
        >>> engine.fitness(my_fitness_func)
        """
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
        Set the fitness function for regression problems.
        This is a convenience method that configures the engine with a regression fitness function based on the
        provided features and targets. This method is fully compatible with the polars, pandas, and numpy libraries for data handling.

        Using this method will automatically set the engine's objective to minimization, as regression problems typically involve minimizing a loss function.

        Accepts:
        - (features, targets)
        - a DataFrame (polars / pandas) with optional target and feature column specifications.
        Args:
            features: The input features for the regression problem. Can be a tuple of (features, targets) or a DataFrame.
            targets: The target values for the regression problem. Required if features is not a tuple.
            *,
            target: The name of the target column if features is a DataFrame.
            feature_cols: The names of the feature columns if features is a DataFrame.
            loss: The loss function to use for regression (e.g., "mse", "mae").
            batch: Whether to compute fitness in batches (useful for large datasets).
        Returns:
            Engine: The engine instance with the regression fitness function set.

        Example:
        ---------
        >>> import polars as pl
        >>> df = pl.DataFrame({
        ...     "feature1": [1.0, 2.0, 3.0],
        ...     "feature2": [4.0, 5.0, 6.0],
        ...     "target": [7.0, 8.0, 9.0]
        ... })
        >>> engine.regression(df, target="target", feature_cols=["feature1", "feature2"], loss="mse")

        --- or simply ---
        >>> features = [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]
        >>> targets = [7.0, 8.0, 9.0]
        >>> engine.regression(features, targets, loss="mse")
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
        """
        Set the selection strategies for the engine.
        This method allows you to specify the selection strategies for both offspring and survivors, as well as
        the fraction of offspring to survivors in the next generation. If no parameters are
        provided, the engine will use its default selection strategies and offspring fraction.

        Defaults:
        - Offspring Selector: rd.Select.tournament(k=3)
        - Survivor Selector: rd.Select.roulette()
        - Offspring Fraction: 0.8 (80% offspring, 20% survivors)

        Args:
            offspring: The selection strategy to use for selecting offspring. If None, the default will be used.
            survivor: The selection strategy to use for selecting survivors. If None, the default will be used.
            frac: The fraction of offspring to survivors in the next generation. Must be between 0 and 1. If None, the default will be used.

        Returns:
            Engine: The engine instance with the selection strategies set.

        Example:
        ---------
        >>> offspring_selector = rd.Select.tournament(k=5)
        >>> survivor_selector = rd.Select.boltzmann(temp=4.0)
        >>> engine.select(offspring=offspring_selector, survivor=survivor_selector, frac=0.7)  # 70% offspring, 30% survivors
        """
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
        """
        Set the alteration operators for the engine.

        This method allows you to specify one or more alteration operators
        (e.g., mutation, crossover) to be applied during the evolution process.
        If no operators are provided, the engine will use its default alteration strategies.

        Defaults:
        - Crossover: rd.Cross.uniform(rate=0.5)
        - Mutation: rd.Mutate.uniform(rate=0.1)

        Args:
            *alters: One or more alteration operators to apply during evolution.
        Returns:
            Engine: The engine instance with the alteration operators set.

        Example:
        ---------
        >>> engine.alters(rd.Cross.multipoint(0.5, 2), rd.Mutate.uniform(0.1))
        """
        self._builder.set_alters(list(alters))
        return self

    def diversity(
        self, diversity: DistanceBase, species_threshold: float = 1.5
    ) -> Engine[G, T]:
        """Set the diversity strategy for the engine."""
        self._builder.set_diversity(diversity, species_threshold)
        return self

    def limit(self, *limits: LimitBase) -> Engine[G, T]:
        """Set the limits for the engine."""
        self._builder.set_limits(list(limits))
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

    def age(
        self, max_phenotype_age: int | None = None, max_species_age: int | None = None
    ) -> Engine[G, T]:
        """Set the aging parameters for the engine."""
        if max_phenotype_age is not None:
            if max_phenotype_age <= 0:
                raise ValueError("Max phenotype age must be greater than 0.")
            self._builder.set_max_age(max_phenotype_age)

        if max_species_age is not None:
            if max_species_age <= 0:
                raise ValueError("Max species age must be greater than 0.")
            self._builder.set_max_species_age(max_species_age)

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
        self._builder.set_objective(obj if isinstance(obj, str) else list(obj))
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
