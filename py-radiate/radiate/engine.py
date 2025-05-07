from typing import Any, Callable, Dict, Union, List
from .problem import Problem
from .genome import Genome
from .selector import Selector, TournamentSelector, RouletteSelector
from .alterer import Alterer, UniformCrossover, UniformMutator
from ._typing import GeneType
from .codex import FloatCodex

from radiate.radiate import PyEngine, PyEngineParam, PyEngineBuilder

class Engine:

    def __init__(self, 
                 codex: FloatCodex,
                 fitness_func: Callable[[Any], Any],
                 offspring_selector: Selector = None,
                 survivor_selector: Selector = None,
                 alters: None | Alterer | List[Alterer] = None,):
        self.codex = codex
        self.fitness_func = fitness_func

        survivor_selector = survivor_selector or TournamentSelector(k=3)
        offspring_selector = offspring_selector or RouletteSelector()
        alters = alters or [UniformCrossover(), UniformMutator()]

        self.builder = PyEngineBuilder(
            objectives=['min'],
            survivor_selector=survivor_selector or TournamentSelector(k=3),
            offspring_selector=offspring_selector or RouletteSelector(),
            alters=alters or [UniformCrossover(), UniformMutator()],
            population_size=100,
        )

        if isinstance(self.codex, FloatCodex):
            self.gene_type = GeneType.FLOAT
        else:
            raise TypeError(f"Codex type {type(self.codex)} is not supported.")

    
    def __repr__(self):
        return f"EngineTest(codex={self.gene_type})"
        

    

 
# class Engine:

#     def __init__(self,
#                  genome: None | Genome = None,
#                  problem: None | Problem | Callable[[Any], Any] = None,
#                  survivor_selector: None | Selector = None,
#                  offspring_selector: None | Selector = None,
#                  alters: None | Alterer | List[Alterer] = None,
#                  objective: str | List[str] = None,
#                  population_size: int = 100,
#                    **kwargs):
#         """Initialize the engine with genome and problem."""
#         self._genome = genome
#         self._survivor_selector: Selector = survivor_selector if survivor_selector is not None else Selector.tournament(k=3)
#         self._offspring_selector: Selector = offspring_selector if offspring_selector is not None else Selector.roulette()
#         self._alters: List[PyEngineParam] = Engine.__build_alterers(genome, alters)
#         self._objective = objective

#         if problem is not None:
#             if isinstance(problem, Problem):
#                 self._fitness_func = problem.fitness_fn
#             elif callable(problem):
#                 self._fitness_func = problem
#             else:
#                 raise TypeError("Problem must be a Problem instance or a callable function.")
        
#         if self._genome is None:
#             raise ValueError("Genome must be provided.")
        
#         if self._genome.gene_type is GeneType.FLOAT:
#             self._engine = self.__build_float_engine()


#     def __build_alterers(genome: None | Union[Genome, Dict[str, Any]] = None, 
#                          alters: None | Alterer | List[Alterer] = None) -> List[PyEngineParam]:
#         """Build a list of alterers."""
#         result = alters if alters is not None else []
        
#         for alter in result:
#             if not alter.is_valid(genome.gene_type):
#                 raise TypeError(f"Alterer {alter} is not valid for genome type {genome.gene_type}.")
            
#         return [alter.params for alter in result]
            

#     def run(self, num_generations=1000):
#         """Run the engine for a number of generations."""
#         self._engine.run(num_generations)


#     def __build_float_engine(self) -> PyEngine:
#         """Build a float engine."""
#         return PyEngine.try_build_float_engine(
#             num_genes=self._genome.num_genes,
#             num_chromosomes=self._genome.num_chromosomes,
#             objective=self._objective,
#             range=(self._genome.min_value, self._genome.max_value),
#             bounds=(self._genome.min_bound, self._genome.max_bound),
#             fitness_fn=self._fitness_func,
#             survivor_selector=self._survivor_selector.selector,
#             offspring_selector=self._offspring_selector.selector,
#             alters=self._alters
#         )
    

#     def __iter__(self):
#         """Iterate over the engine."""
#         return self
    
#     def __next__(self):
#         """Get the next generation."""
#         return self._engine.next()

