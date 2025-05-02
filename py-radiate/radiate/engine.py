
from dataclasses import asdict
from typing import Any, Callable, Dict, Union
from radiate.problem import Problem
from radiate.selector import Selector
from radiate.genome import Genome

from radiate.radiate import  PyEngine

 
class Engine:

    def __init__(self,
                 genome=None | Union[Genome, Dict[str, Any]],
                 problem=None | Problem | Callable[[Any], Any],
                 survivor_selector=None | Selector,
                 offspring_selector=None | Selector,
                 fitness_func = None,
                   **kwargs):
        
        
        self.config = genome
        if isinstance(self.config, Genome):
            self.config = asdict(self.config)
        else:
            self.config = self.config        

        if fitness_func is not None:
            self.config['fitness_func'] = fitness_func
        if survivor_selector is not None:
            self.config['survivor_selector'] = survivor_selector
        if offspring_selector is not None:
            self.config['offspring_selector'] = offspring_selector
        if problem is not None:
            if isinstance(problem, Problem):
                self.config['fitness_func'] = problem.fitness_fn
            elif callable(problem):
                self.config['fitness_func'] = problem
            else:
                raise TypeError("Problem must be a Problem instance or a callable function.")

        self._engine = PyEngine.try_build_float_engine(
            num_genes=self.config.get('num_genes', 1),
            num_chromosomes=self.config.get('num_chromosomes', 1),
            range=(self.config.get('min_value', 0.0), self.config.get('max_value', 1.0)),
            bounds=(self.config.get('min_bound', None), self.config.get('max_bound', None)),
            fitness_fn=self.config.get('fitness_func', None),
        )
        
        print(self.config)

    def run(self, num_generations=1000):
        """Run the engine for a number of generations."""
        self._engine.run(num_generations)

