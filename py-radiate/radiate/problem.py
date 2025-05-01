# problem.py
from abc import ABC, abstractmethod
from typing import Generic, TypeVar, Callable, Sequence

T = TypeVar('T')
FitnessType = TypeVar('FitnessType', float, Sequence[float])

class Problem(Generic[T, FitnessType]):
    """Base class for optimization problems"""
    
    def __init__(self, fitness_func: Callable[[Sequence[T]], FitnessType]):
        self.fitness_func = fitness_func
        
    @abstractmethod
    def evaluate(self, solution: Sequence[T]) -> FitnessType:
        """Evaluate a solution"""
        pass

class SingleObjectiveProblem(Problem[T, float]):
    """Single objective optimization problem"""
    
    def evaluate(self, solution: Sequence[T]) -> float:
        return self.fitness_func(solution)

class MultiObjectiveProblem(Problem[T, Sequence[float]]):
    """Multi-objective optimization problem"""
    
    def evaluate(self, solution: Sequence[T]) -> Sequence[float]:
        return self.fitness_func(solution)