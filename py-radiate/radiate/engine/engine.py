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

        The fitness function is a critical component of the genetic algorithm, as it evaluates the quality of each
        individual in the population and guides the selection process. This method allows you to specify a
        custom fitness function that takes a decoded genome as input and returns a fitness score.
        The fitness function can be provided as a simple callable (e.g., a lambda function or a regular function) or as
        an instance of a class that inherits from FitnessBase, which allows for more complex fitness evaluation strategies (e.g., batch evaluation, novelty search).

        The fitness function need to accept the decoded genome as input. For example, if we configure the engine as:

        >>> import radiate as rd
        >>> ...
        >>> def my_fitness_func(genome: list[float]) -> float:
        ...     return sum(genome)  # Example fitness function that sums the genome values
        >>> ...
        >>> engine = rd.Engine.float(shape=3, init_range=(-10, 10)).fitness(my_fitness_func)

        The fitness function `my_fitness_func` should be defined to accept a list of 3 floats (or a numpy array if use_numpy=True):
        On the otherhand, if we configure the engine as such:

        >>> import radiate as rd
        >>> ...
        >>> def my_fitness_func(genome: list[list[int]]) -> int:
        ...     return sum(sum(g) for g in genome)  # Example fitness function that sums all values in a 2D genome
        >>> ...
        >>> engine = rd.Engine.int(shape=[5, 5, 5, 5, 5], init_range=(-10, 10)).fitness(my_fitness_func)

        The fitness function `my_fitness_func` should be defined to accept a list of 5 lists of 5 integers (or a numpy array if use_numpy=True):

        Args:
            fitness_func: A callable that takes a decoded genome and returns a fitness score, or an instance of FitnessBase.

        Returns:
            Engine: The engine instance with the fitness function set.
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

        The loss parameter accepts four common loss functions for regression:
        - **mse**: Mean Squared Error
        - **mae**: Mean Absolute Error
        - **cross_entropy**: Cross-Entropy Loss (for classification problems, but can be used in regression with appropriate encoding)
        - **diff**: A simple difference loss that calculates the absolute difference between predicted and target values.

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
        Create a simple Graph engine to fit a cubic function using mean squared error regression
        ---------
        >>> # first we'll use just normal lists for our features and targets.
        >>> import radiate as rd
        >>> ...
        >>> features = [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]
        >>> targets = [7.0, 8.0, 9.0]
        >>> ...
        >>> base_engine = (
        ...     rd.Engine.graph(
        ...         shape=(2, 1), # <- notice how the shape of the graph is (2, 1) to accommodate our 2 features and 1 output
        ...         vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        ...         edge=rd.Op.weight(),
        ...         output=rd.Op.linear(),
        ...     )
        ...     .regression(features, targets, loss="mse") # <- we directly pass our features/targets to the regression method. The engine is now also configured to minimize the mean squared error between the graph's output and our targets.
        ...     .alters(
        ...         rd.Cross.graph(0.05, 0.5),
        ...         rd.Mutate.op(0.07, 0.05),
        ...         rd.Mutate.graph(0.1, 0.1, False),
        ...     )
        ... )

        Now, we can also use a DataFrame to directly specify our features and targets, which can be more convenient for
        larger datasets and also allows us to easily specify which columns are features and which is the target.

        We'll also switch up the error function below to use mean average error instead of mean squared error just to show flexibility.

        >>> import polars as pl
        >>> df = pl.DataFrame({
        ...     "feature1": [1.0, 2.0, 3.0],
        ...     "feature2": [4.0, 5.0, 6.0],
        ...     "target": [7.0, 8.0, 9.0]
        ... })
        >>> base_engine.regression(df, target="target", feature_cols=["feature1", "feature2"], loss="mae")

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
        >>> import radiate as rd
        >>> target_str = "some target string"
        >>> engine = (
        ...     rd.Engine.char(len(target_str))
        ...     .fitness(lambda genome: sum(1 for g, t in zip(genome, target_str) if g == t))
        ...     .select(
        ...         offspring=rd.Select.tournament(k=5), # <- tournament selection with k=5 for offspring
        ...         survivor=rd.Select.boltzmann(temp=4.0), # <- boltzmann selection with temperature 4.0 for survivors
        ...         frac=0.7 # <- 70% offspring, 30% survivors in the next generation
        ...     )
        ... )
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
        Create an engine that evolves a 1D np.array of UInt8 values and uses multi-point crossover and uniform mutation
        >>> import radiate as rd
        >>> n_queens = 32
        >>> engine = (
        ...    rd.Engine.int(n_queens, init_range=(0, n_queens), use_numpy=True, dtype=rd.UInt8)
        ...    .fitness(my_fitness_fn)
        ...    .minimizing()
        ...    .select(offspring=rd.TournamentSelector(k=3))
        ...    .alters(
        ...        rd.MultiPointCrossover(0.75, 2), # <- multi-point crossover with 75% rate and 2 crossover points
        ...        rd.UniformMutator(0.05), # <- uniform mutation with 5% mutation rate
        ...    )
        ... )
        """
        self._builder.set_alters(list(alters))
        return self

    def diversity(
        self, diversity: DistanceBase, species_threshold: float = 1.5
    ) -> Engine[G, T]:
        """
        Set the diversity measure and species threshold for speciation in the engine.

        This method allows you to specify a distance-based diversity measure to promote genetic diversity in the population,
        as well as a species threshold that determines how individuals are grouped into species based on their genetic distance.
        The default for this is None, so without specifiying a distance (diversity) measure, the engine will not perform speciation.
        If a diversity measure is provided, the engine will use it to calculate genetic distances between individuals
        and group them into species based on the specified threshold. It should be noted that this increases the computational
        overhead of the engine, so it is recommended to use this feature when maintaining diversity is a concern for the problem at hand.

        Defaults:
        - Diversity Measure: None (no speciation)
        - Species Threshold: 1.5
        Args:
            diversity: A distance-based diversity measure to promote genetic diversity.
            species_threshold: A threshold for grouping individuals into species based on genetic distance. Must be greater than 0.
        Returns:
            Engine: The engine instance with the diversity measure and species threshold set.

        Example:
        ---------
        Create an engine that evolves a 2x2 matrix (list[list[float]]) of floats and uses Euclidean distance for speciation with a threshold of .7
        >>> import radiate as rd
        >>> engine = (
        ...     rd.Engine.float(shape=[2, 2], init_range=(0.0, 10.0))
        ...     .fitness(my_fitness_function)
        ...     .diversity(rd.Dist.euclidean(), species_threshold=0.7) # <- use Euclidean distance for speciation with a threshold of 0.7
        ... )
        """
        self._builder.set_diversity(diversity, species_threshold)
        return self

    def limit(self, *limits: LimitBase) -> Engine[G, T]:
        """
        Set the limits for the engine.

        This method allows you to specify one or more limits that will be applied during the evolution process. Limits can be applied here,
        or in the call to the run method. If limits are set in both places, they will be combined and all limits will be considered during the engine's
        execution.

        If no limits are provided anywhere, the engine will raise an exception when run, as at least one limit is required to determine
        when the engine should stop evolving. The engine will stop when the first limit is reached, so if multiple limits are provided,
        the engine will monitor all of them and stop as soon as any one of them is reached.

        Options:
        - rd.Limit.score(threshold): Stop when a solution reaches a fitness score threshold.
        - rd.Limit.generation(count): Stop after a certain number of generations.
        - rd.Limit.seconds(time): Stop after a certain amount of time has elapsed.
        - rd.Limit.convergence(window, epsilon): Stop when the population has converged based on a specified window of generations and convergence threshold.
        - rd.Limit.metric(metric_name, lambda metric: bool): Stop when a custom metric function returns True.

        Args:
            *limits: One or more limits to apply during evolution.

        Returns:
            Engine: The engine instance with the limits set.

        Example:
        ---------
        >>> import radiate as rd
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(shape=10, init_range=(0.0, 1.0))
        ...     .fitness(my_fitness_function)
        ...     .minimizing()
        ...     .limit(
        ...         rd.Limit.score(0.001), # <- stop when a solution reaches a fitness score of 0.001 or better
        ...         rd.Limit.generation(1000), # <- stop after 1000 generations
        ...         rd.Limit.seconds(60), # <- stop after 60 seconds
        ...         rd.Limit.convergence(window=50, epsilon=0.0001), # <- stop when the population has converged with a window of 50 generations and a convergence threshold of 0.0001
        ...         rd.Limit.metric("evaluation_count", lambda metric: metric.sum() >= 1000) # <- stop when the engine has evaluated at least 1000 solutions
        ...     )
        ... )
        >>> ...
        >>> result = engine.run() # <- run the engine with the specified limits. The engine will stop when any of the limits are reached.
        """
        self._builder.set_limits(list(limits))
        return self

    def size(self, size: int) -> Engine[G, T]:
        """
        Set the population size for the engine.

        This method allows you to specify the number of individuals in the population for each generation.
        A larger population size can increase the genetic diversity and potentially lead to better solutions, but
        it also increases the computational cost of each generation. Conversely, a smaller population size can
        reduce computational cost but may lead to premature convergence on suboptimal solutions.
        The optimal population size can depend on the specific problem being solved, so it may require some experimentation to find the best value.

        Args:
            size: The number of individuals in the population for each generation. Must be greater than 0.
        Returns:
            Engine: The engine instance with the population size set.

        Example:
        ---------
        >>> import radiate as rd
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(shape=5, init_range=(0.0, 1.0))
        ...     .fitness(my_fitness_function)
        ...     .size(200) # <- set the population size to 200 individuals per generation
        ... )
        """
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")
        self._builder.set_population_size(size)
        return self

    def age(
        self, max_phenotype_age: int | None = None, max_species_age: int | None = None
    ) -> Engine[G, T]:
        """
        Set the maximum age for phenotypes and species in the engine.

        This method allows you to specify the maximum age for both phenotypes (individuals) and species (if speciation is configured) in the population.
        Age limits can help ensure that the population does not become stagnant by removing older individuals or species that may no
        longer be contributing to the evolution process. When an individual's age exceeds the specified maximum phenotype age,
        it will be removed from the population & replaced by a newly encoded one. Similarly, when a species' age exceeds
        the specified maximum species age, it will be removed from the population & it's members will not be allowed to participate
        in crossover and mutation. Setting these limits can promote diversity and encourage the exploration of new solutions,
        but setting them too low may result in the loss of potentially valuable genetic material.
        It may require some experimentation to find the optimal age limits for a given problem.

        Defaults:
        - **max_phenotype_age**: 20
        - **max_species_age**: 20

        Args:
            max_phenotype_age: The maximum age for phenotypes (individuals) in the population. Must be greater than 0. If None, the default will be used.
            max_species_age: The maximum age for species in the population. Must be greater than 0. If None, the default will be used.
        Returns:
            Engine: The engine instance with the age limits set.

        Example:
        ---------
        >>> import radiate as rd
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(shape=5, init_range=(0.0, 1.0))
        ...     .fitness(my_fitness_function)
        ...     .age(max_phenotype_age=30, max_species_age=50) # <- set the maximum phenotype age to 30 generations and the maximum species age to 50 generations
        ... )
        """
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
        """
        Set the initial population for the engine.

        This method allows you to specify an initial population of decoded genomes to start the evolution process.
        Providing a custom initial population can be useful for seeding the engine with known good solutions or for
        continuing evolution from a previous run. The population should be provided as an
        instance of the Population class, which can be created from a list of encoded genomes.

        Args:
            population: An instance of the Population class containing the initial population of decoded genomes.
        Returns:
            Engine: The engine instance with the initial population set.

        Example:
        ---------
        >>> import radiate as rd
        >>> ...
        >>> codec = rd.FloatCodec.vector(5, init_range=(-5.12, 5.12))
        >>> population = rd.Population(rd.Phenotype(codec.encode()) for _ in range(107))
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(5, init_range=(-5.12, 5.12))
        ...     .fitness(fitness_fn)
        ...     .minimizing()
        ...     .population(population)
        ...     .alters(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.01))
        ... )
        """
        self._builder.set_population(population)
        return self

    def minimizing(self) -> Engine[G, T]:
        """
        Set the objectives to minimize.

        In many optimization problems, the goal is to minimize a fitness function (e.g., minimize error, cost, or distance).
        This method configures the engine to treat the fitness function as a minimization objective, meaning that lower fitness scores will be considered better solutions.
        If you want to maximize the fitness function instead (e.g., maximize accuracy, profit, or reward), you can use the maximizing() method.

        The default for the Engine is to maximize the fitness function, so if you want to minimize, you need to explicitly call this method.

        Example:
        ---------
        Lets create an engine that minimizes the sum of two bit chromosomes. The first chromosome we want to be all True while the second
        chromosome we want to be all False, so the optimal solution would be:

        ```
        [
            [True, True, True, True, True],
            [False, False, False, False, False]
        ]
        ```

        with a fitness score of 0 (the minimum possible score for this problem).
        >>> import radiate as rd
        >>> ...
        >>> def fit(x: list[list[bool]]) -> int:
        ...     sum_one = sum(1 for bit in x[0] if bit)
        ...     sum_two = sum(1 for bit in x[1] if not bit)
        ...     return sum_one + sum_two
        >>> ...
        >>> # Two chromosomes both with 5 genes in it.
        >>> # We want the first chromosome to be all ones and the second chromosome to be all zeros.
        >>> engine = (
        ...     rd.Engine.bit([5, 5])
        ...     .fitness(fit)
        ...     .minimizing() # <- we want to minimize the fitness score
        ... )
        >>> engine.run(rd.Limit.score(0)) # <- run the engine with a score limit of 0, which is the optimal solution for this problem. The engine will evolve until it finds the optimal solution with a fitness score of 0.
        """
        self._builder.set_objective("min")
        return self

    def maximizing(self) -> Engine[G, T]:
        """
        Set the objectives to maximize.

        In many optimization problems, the goal is to maximize a fitness function (e.g., maximize accuracy, profit, or reward).
        This method configures the engine to treat the fitness function as a maximization objective, meaning that higher fitness scores will be considered better solutions.
        If you want to minimize the fitness function instead (e.g., minimize error, cost, or distance), you can use the minimizing() method.

        The engine's default is to maximize the fitness function, so in reality, theres no reason to call this method unless you
        have previously called minimizing() and want to switch back to maximizing. This method is pretty much only included here for
        completeness and to provide a more fluent API for configuring the engine's optimization objective.

        Example:
        ---------
        Lets create an engine that maximizes the number of ones in a bit chromosome. The optimal
        solution would be a chromosome of all ones with a fitness score of 10 (the maximum possible score for this problem).
        >>> import radiate as rd
        >>> ...
        >>> def fit(x: list[bool]) -> int:
        ...     return sum(1 for bit in x if bit)
        >>> ...
        >>> engine = (
        ...     rd.Engine.bit(10)
        ...     .fitness(fit)
        ...     .maximizing() # <- we want to maximize the fitness score (again, we don't need to do this since maximizing is the default)
        ... )
        >>> engine.run(rd.Limit.score(10)) # <- run the engine with a score limit of 10, which is the optimal solution for this problem. The engine will evolve until it finds the optimal solution with a fitness score of 10.
        """
        self._builder.set_objective("max")
        return self

    def objective(
        self, *obj: str, front_range: tuple[int, int] | None = None
    ) -> Engine[G, T]:
        """
        Set the optimization objectives for the engine.

        This allows you to specify one or more optimization objectives for the engine, which can be either rd.MIN for minimization or rd.MAX for maximization.
        This method is mainly used for configuring multi-objective optimization problems, where you may
        want to optimize for multiple criteria simultaneously (e.g., minimize cost while maximizing performance).

        It also allows you to simultaineously set the front range for the Pareto front in multi-objective optimization,
        which determines how many solutions are kept in the Pareto front at each generation. The min of the front range determines the
        minimum number of individuals that will be kept in the Pareto front, while the max of the front range determines the limit
        on how many _can_ be kept - when the front reaches the max it will filter down to the min.

        If this method is called and no objectives are supplied, the engine will raise an exception.
        If this method is not called at all, the engine will default to maximizing a single objective (the fitness function).

        Args:
            *obj: One or more optimization objectives (rd.MIN for minimization, rd.MAX for maximization). For multi-objective optimization, provide multiple objectives in the desired order of importance.
            front_range: Optional tuple specifying the range for the Pareto front in multi-objective optimization. If provided, it should be a tuple of two integers (min, max).
        Returns:
            Engine: The engine instance with the optimization objectives set.

        Example:
        ---------
        Lets create an engine that optimizes for two objectives simultaneously: we want to minimize the number
        of ones in the first chromosome while maximizing the number of ones in the second chromosome. This is a
        very very simple multi-objective optimization problem, but it serves to illustrate how to use
        the objective() method to configure multiple objectives and the Pareto front range.

        >>> import radiate as rd
        >>> ...
        >>> def fit(genome: list[list[bool]]) -> tuple[int, int]:
        ...     obj1 = sum(1 for bit in genome[0] if bit)
        ...     obj2 = sum(1 for bit in genome[1] if bit)
        ...     return obj1, obj2
        >>> ...
        >>> engine = (
        ...     rd.Engine.bit([5, 5])
        ...     .fitness(fit)
        ...     .objective(rd.MIN, rd.MAX, front_range=(100, 150)) # <- we want to minimize the number of ones in the first chromosome and maximize the number of ones
        ...     # in the second chromosome, and we want to keep between 100 and 150 solutions in the Pareto front at each generation
        ... )
        """
        self._builder.set_objective(obj if isinstance(obj, str) else list(obj))
        if front_range is not None:
            self._builder.set_front_range(*front_range)
        return self

    def front_range(self, min: int, max: int) -> Engine[G, T]:
        """
        Set the front range for the Pareto front in multi-objective optimization.

        This method allows you to specify the front range for the Pareto front when performing multi-objective optimization.
        The front range determines how many solutions are kept in the Pareto front at each generation. The minimum of
        the front range specifies the minimum number of individuals that will be kept in the Pareto front, while the
        maximum of the front range specifies the limit on how many individuals can be kept.
        When the number of solutions in the Pareto front reaches the maximum, it will filter down to the minimum.

        You can also do this while calling the .objective(..., front_range=(min, max)) method, which allows you to set the
        optimization objectives and the front range simultaneously. This is kinda just a helper method for configuring the front
        range if you want to set it separately from the objectives for some reason.

        Args:
            min: The minimum number of individuals to keep in the Pareto front. Must be greater than 0.
            max: The maximum number of individuals to keep in the Pareto front. Must be greater than or equal to min.
        Returns:
            Engine: The engine instance with the front range set.

        Example:
        ---------
        Lets create an engine that optimizes for two objectives simultaneously and set the front range separately from the objectives. Again,
        this is an extremely contrived example just to illustrate how to use the front_range() method.
        >>> import radiate as rd
        >>> ...
        >>> def fit(genome: list[list[bool]]) -> tuple[int, int]:
        ...     obj1 = sum(1 for bit in genome[0] if bit)
        ...     obj2 = sum(1 for bit in genome[1] if bit)
        ...     return obj1, obj2
        >>> ...
        >>> engine = (
        ...     rd.Engine.bit([5, 5])
        ...     .fitness(fit)
        ...     .objective(rd.MIN, rd.MAX) # <- we want to minimize the number of ones in the first chromosome and maximize the number of ones in the second chromosome
        ...     .front_range(100, 150) # <- we want to keep between 100 and 150 solutions in the Pareto front at each generation
        ... )
        """
        self._builder.set_front_range(min, max)
        return self

    def parallel(self, num_workers: int | None = None) -> Engine[G, T]:
        """
        Configure the engine to run in parallel using multiple worker threads.

        The default for the engine is to run synchronously on a single thread. But, if you are able to take advantage of
        parallelism (e.g., if your fitness function is computationally expensive and can be evaluated in parallel across multiple individuals),
        you can use this method to configure the engine to run in parallel using multiple worker threads.

        If num_workers is not provided, the engine will use rayon's global thread pool, which automatically determines the optimal
        number of worker threads based on the available CPU cores. If num_workers is provided, it will create a
        fixed-size thread pool with the specified number of worker threads.

        If your python version does not support free-threaded python or the GIL is enabled, the engine will raise an exception when the engine
        builds and will not run. In this case, you can either disable the GIL (if your python implementation allows it) or use a different python
        interpreter that supports free-threaded python. There is one important caveat to be aware of: regression problems run in pure rust,
        so if you are running a regression problem with Graphs or Trees, you can use this feature regardless of free-threaded python or GIL support.

        Args:
            num_workers: The number of worker threads to use for parallel execution.
        Returns:
            Engine: The engine instance configured for parallel execution.

        Example:
        ---------
        Lets create an engine that evolves a population of 1000 individuals, where each individual's fitness evaluation is
        computationally expensive. We can configure the engine to run in parallel using multiple worker threads to
        speed up the evaluation process.
        >>> import radiate as rd
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(shape=10, init_range=(0.0, 1.0))
        ...     .fitness(my_expensive_fitness_function)
        ...     .parallel(num_workers=8) # <- configure the engine to use a fixed size thread pool with 8 worker threads for parallel execution
        ... )
        """
        executor = (
            Executor.WorkerPool()
            if num_workers is None
            else Executor.FixedSizedWorkerPool(num_workers)
        )

        self._builder.set_executor(executor)
        return self

    def subscribe(self, event_handler: Subscriber | None = None) -> Engine[G, T]:
        """
        Subscribe to engine events with a custom event handler.

        This method allows you to subscribe to various events that occur during the engine's execution by providing a custom event handler. There
        are two main ways to subscribe to events:

        1.) By subclassing the rd.EventHandler base class & overriding the 'on_event' method to handle specific events you are interested in.

        2.) By providing a simple function that takes an event object as input and handles it accordingly. This will subscribe the ALL engine events

        By subclassing the rd.EventHandler base class, you can also specify which events you want to listen to such as:

        - All: Listen to all events emitted by the engine.
        - Start: Listen to the event emitted when the engine starts running.
        - Stop: Listen to the event emitted when the engine stops running.
        - Epoch_Start: Listen to the event emitted at the start of each epoch.
        - Epoch_Complete: Listen to the event emitted at the end of each epoch.
        - Engine_Improvement: Listen to the event emitted whenever the engine finds a new best solution that improves upon the previous best solution.

        Args:
            event_handler: One or more instances of a custom event handler that subclasses rd.EventHandler,
            or one or more simple functions that takes an event object as input. If None, the engine will not subscribe to any events.
        Returns:
            Engine: The engine instance with the event handler(s) subscribed to engine events.

        Example:
        ---------
        >>> import radiate as rd
        >>> ...
        >>> class MyEventHandler(rd.EventHandler):
        ...     def __init__(self):
        ...         super().__init__(rd.EventType.ENGINE_IMPROVEMENT) # <- we only want to listen to engine improvement events
        ... ...
        ...     def on_event(self, event: rd.EngineEvent) -> None:
        ...         print(event)
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(shape=5, init_range=(0.0, 1.0))
        ...     .fitness(my_fitness_function)
        ...     .subscribe(MyEventHandler()) # <- subscribe to engine events with our custom event handler that listens to engine improvement events and prints them out
        ... )
        """
        self._builder.set_subscribers(event_handler)
        return self

    def generation(self, generation: Generation[T] | None) -> Engine[G, T]:
        """
        Set the initial Generation for the engine.

        this is mainly intended for use cases where an existing Generation exists and you want to continue evolving from that point,
        or if you have a specific Generation that you want to use as the starting point for evolution.
        If no Generation is provided, the engine will start with a newly initialized population based on the configuration of the builder.

        Args:
            generation: An instance of the Generation class to use as the initial generation for the engine.
        Returns:
            Engine: The engine instance with the initial generation set.

        Example:
        ---------
        >>> import radiate as rd
        >>> ...
        >>> # Let's say we have an existing generation that we want to continue evolving from.
        >>> existing_generation = rd.Generation(...) # <- create or load an existing generation
        >>> ...
        >>> engine = (
        ...     rd.Engine.float(shape=5, init_range=(0.0, 1.0))
        ...     .fitness(my_fitness_function)
        ...     .generation(existing_generation) # <- set the initial generation for the engine to our existing generation
        ... )
        """
        self._builder.set_generation(generation)
        return self
