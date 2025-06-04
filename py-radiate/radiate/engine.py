from typing import Any, Callable, List, Tuple
from .selector import SelectorBase, TournamentSelector, RouletteSelector
from .alterer import AlterBase, UniformCrossover, UniformMutator
from .diversity import DiversityBase, HammingDistance, EuclideanDistance
from .codec import CodecBase
from .limit import LimitBase
from .generation import Generation


from radiate.radiate import PyEngineBuilder, PyObjective, PySubscriber


class GeneticEngine:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: CodecBase,
        fitness_func: Callable[[Any], Any],
        offspring_selector: SelectorBase | None = None,
        survivor_selector: SelectorBase | None = None,
        alters: None | AlterBase | List[AlterBase] = None,
        diversity: None | DiversityBase = None,
        population_size: int = 100,
        offspring_fraction: float = 0.8,
        max_phenotype_age: int = 20,
        max_species_age: int = 20,
        species_threshold: float = 1.5,
        objectives: str | List[str] = ["min"],
        num_threads: int = 1,
        front_range: Tuple[int, int] | None = (800, 900),
    ):
        self.engine = None

        survivor_selector = self.__get_params(
            survivor_selector or TournamentSelector(k=3)
        )
        offspring_selector = self.__get_params(offspring_selector or RouletteSelector())
        alters = self.__get_params(alters or [UniformCrossover(), UniformMutator()])
        diversity = self.__get_params(diversity, allow_none=True)
        objectives = self.__get_objectives(objectives)
        front_range = self.__get_front_range(front_range)

        codec = self.__get_codec(codec)
        fitness_func = fitness_func

        self.builder = PyEngineBuilder(
            fitness_func,
            codec,
            population_size=population_size,
            offspring_fraction=offspring_fraction,
            objective=objectives,
            front_range=front_range,
            num_threads=num_threads,
            max_phenotype_age=max_phenotype_age,
            max_species_age=max_species_age,
            species_threshold=species_threshold,
            alters=alters,
            offspring_selector=offspring_selector,
            survivor_selector=survivor_selector,
            diversity=diversity,
        )

    def run(
        self, limits: LimitBase | List[LimitBase] | None, log: bool = False
    ) -> Generation:
        """Run the engine with the given limits."""
        if limits is not None:
            self.limits(limits)

        self.engine = self.builder.build()
        return Generation(self.engine.run(log=log))

    def population_size(self, size: int):
        """Set the population size."""
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")
        self.builder.set_population_size(size)

    def survivor_selector(self, selector: SelectorBase):
        """Set the survivor selector."""
        if selector is None:
            raise ValueError("Selector must be provided.")
        self.builder.set_survivor_selector(self.__get_params(selector))

    def offspring_selector(self, selector: SelectorBase):
        """Set the offspring selector."""
        if selector is None:
            raise ValueError("Selector must be provided.")
        self.builder.set_offspring_selector(self.__get_params(selector))

    def alters(self, alters: AlterBase | List[AlterBase]):
        """Set the alters."""
        if alters is None:
            raise ValueError("Alters must be provided.")
        self.builder.set_alters(self.__get_params(alters))

    def limits(self, limits: LimitBase | List[LimitBase]):
        """Set the limits."""
        if limits is None:
            raise ValueError("Limits must be provided.")
        lims = [lim.limit for lim in (limits if isinstance(limits, list) else [limits])]
        self.builder.set_limits(lims)

    def diversity(self, diversity: DiversityBase, species_threshold: float = 1.5):
        """Set the diversity."""
        if diversity is None:
            raise ValueError("Diversity must be provided.")
        self.builder.set_diversity(self.__get_params(diversity, allow_none=True))
        self.builder.set_species_threshold(species_threshold)

    def offspring_fraction(self, fraction: float):
        """Set the offspring fraction."""
        if not (0 < fraction <= 1):
            raise ValueError("Offspring fraction must be between 0 and 1.")
        self.builder.set_offspring_fraction(fraction)

    def max_age(self, max_phenotype_age: int = 20, max_species_age: int = 20):
        """Set the maximum age for phenotypes and species."""
        if max_phenotype_age <= 0 or max_species_age <= 0:
            raise ValueError("Maximum age must be greater than 0.")
        self.builder.set_max_phenotype_age(max_phenotype_age)
        self.builder.set_max_species_age(max_species_age)

    def minimizing(self):
        """Set the objectives."""
        self.builder.set_objective(PyObjective.min())

    def maximizing(self):
        """Set the objectives."""
        self.builder.set_objective(PyObjective.max())

    def multi_objective(
        self, objectives: List[str], front_range: Tuple[int, int] | None = None
    ):
        """Set the objectives for a multiobjective problem"""
        if not isinstance(objectives, list) or not all(
            obj in ["min", "max"] for obj in objectives
        ):
            raise ValueError("Objectives must be a list of 'min' or 'max'.")

        self.builder.set_objective(self.__get_objectives(objectives))
        self.builder.set_front_range(self.__get_front_range(front_range))

    def num_threads(self, num_threads: int):
        """Set the number of threads."""
        if num_threads <= 0:
            raise ValueError("Number of threads must be greater than 0.")
        self.builder.set_num_threads(num_threads)

    def subscribe(
        self, event_handler: List[Callable[[Any], None]] | Callable[[Any], None]
    ):
        """Register an event handler."""
        if callable(event_handler):
            self.builder.set_subscribers([PySubscriber(event_handler)])
        elif all(callable(handler) for handler in event_handler):
            self.builder.set_subscribers(
                [PySubscriber(handler) for handler in event_handler]
            )
        else:
            raise TypeError("Event handler must be a callable or a list of callables.")

    def __get_front_range(self, front_range: Tuple[int, int] | None) -> Tuple[int, int]:
        """Get the front range."""
        if front_range is None:
            return (800, 900)
        if not isinstance(front_range, tuple) or len(front_range) != 2:
            raise ValueError("Front range must be a tuple of (min, max).")
        if front_range[0] >= front_range[1]:
            raise ValueError(
                "Minimum front range must be less than maximum front range."
            )
        return front_range

    def __get_objectives(self, objectives: str | List[str]) -> PyObjective:
        """Get the objectives."""
        if objectives is None:
            raise ValueError("Objectives must be provided.")
        if isinstance(objectives, str):
            if objectives not in ["min", "max"]:
                raise ValueError("Objectives must be 'min' or 'max'.")
            return PyObjective([objectives])
        if isinstance(objectives, list):
            for obj in objectives:
                if obj not in ["min", "max"]:
                    raise ValueError("Objectives must be 'min' or 'max'.")
            return PyObjective.multi(objectives)
        raise TypeError(f"Objectives type {type(objectives)} is not supported.")

    def __get_params(
        self,
        value: SelectorBase | DiversityBase | AlterBase | List[AlterBase],
        allow_none: bool = False,
    ) -> List[Any] | None:
        """Get the parameters from the value."""
        if isinstance(value, SelectorBase):
            return value.selector
        if isinstance(value, AlterBase):
            return [value.alterer]
        if isinstance(value, DiversityBase):
            return value.diversity
        if isinstance(value, list):
            if all(isinstance(alter, AlterBase) for alter in value):
                return [alter.alterer for alter in value]

        if allow_none and value is None:
            return None
        else:
            raise TypeError(f"Param type {type(value)} is not supported.")

    def __repr__(self):
        if self.engine is None:
            return f"{self.builder.__repr__()}"
        return f"{self.engine.__repr__()}"

    def __get_codec(self, codec: CodecBase | Callable[[], List[Any]]) -> Any:
        """Get the codec."""
        from .codec import FloatCodec, IntCodec, CharCodec, BitCodec

        if isinstance(codec, FloatCodec):
            return codec.codec
        if isinstance(codec, IntCodec):
            return codec.codec
        if isinstance(codec, CharCodec):
            return codec.codec
        if isinstance(codec, BitCodec):
            return codec.codec

        else:
            raise TypeError(
                f"Codec type {type(codec)} is not supported. "
                "Use FloatCodec, IntCodec, CharCodec, or BitCodec."
            )
