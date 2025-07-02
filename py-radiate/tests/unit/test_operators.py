# """
# Unit tests for Radiate genetic operators.

# These tests focus on individual operator functionality and edge cases.
# """

# import pytest
# import numpy as np
# from typing import List, Tuple

# import radiate as rd
# from radiate import (
#     # Crossovers
#     UniformCrossover, ArithmeticCrossover, BlendCrossover, 
#     IntermediateCrossover, MeanCrossover, MultipointCrossover,
#     PMXCrossover, ShuffleCrossover, SimulatedBinaryCrossover,
#     # Mutators
#     ArithmeticMutator, GaussianMutator, InvertMutator, 
#     ScrambleMutator, SwapMutator, UniformMutator,
#     # Selectors
#     TournamentSelector, RouletteSelector, RankSelector,
#     EliteSelector, BoltzmannSelector, StochasticUniversalSamplingSelector,
#     # Graph operators
#     GraphCrossover, GraphMutator, OperationMutator,
#     # Tree operators
#     TreeCrossover, HoistMutator, SubtreeMutator
# )


# class TestCrossoverOperators:
#     """Unit tests for crossover operators."""
    
#     @pytest.mark.unit
#     def test_uniform_crossover(self, small_int_population):
#         """Test uniform crossover operator."""
#         crossover = UniformCrossover(rate=0.5)
#         parent1 = small_int_population[0]
#         parent2 = small_int_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
#         assert len(offspring) == len(parent2)
    
#     @pytest.mark.unit
#     def test_arithmetic_crossover(self, small_float_population):
#         """Test arithmetic crossover operator."""
#         crossover = ArithmeticCrossover(rate=0.5, alpha=0.5)
#         parent1 = small_float_population[0]
#         parent2 = small_float_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
#         assert len(offspring) == len(parent2)
    
#     @pytest.mark.unit
#     def test_blend_crossover(self, small_float_population):
#         """Test blend crossover operator."""
#         crossover = BlendCrossover(rate=0.5, alpha=0.5)
#         parent1 = small_float_population[0]
#         parent2 = small_float_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
    
#     @pytest.mark.unit
#     def test_intermediate_crossover(self, small_float_population):
#         """Test intermediate crossover operator."""
#         crossover = IntermediateCrossover(rate=0.5)
#         parent1 = small_float_population[0]
#         parent2 = small_float_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
    
#     @pytest.mark.unit
#     def test_mean_crossover(self, small_float_population):
#         """Test mean crossover operator."""
#         crossover = MeanCrossover(rate=0.5)
#         parent1 = small_float_population[0]
#         parent2 = small_float_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
    
#     @pytest.mark.unit
#     def test_multipoint_crossover(self, small_int_population):
#         """Test multipoint crossover operator."""
#         crossover = MultipointCrossover(rate=0.5, points=2)
#         parent1 = small_int_population[0]
#         parent2 = small_int_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
    
#     @pytest.mark.unit
#     def test_pmx_crossover(self, small_int_population):
#         """Test PMX crossover operator."""
#         crossover = PMXCrossover(rate=0.5)
#         parent1 = small_int_population[0]
#         parent2 = small_int_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
    
#     @pytest.mark.unit
#     def test_shuffle_crossover(self, small_int_population):
#         """Test shuffle crossover operator."""
#         crossover = ShuffleCrossover(rate=0.5)
#         parent1 = small_int_population[0]
#         parent2 = small_int_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)
    
#     @pytest.mark.unit
#     def test_simulated_binary_crossover(self, small_float_population):
#         """Test simulated binary crossover operator."""
#         crossover = SimulatedBinaryCrossover(rate=0.5, eta=20.0)
#         parent1 = small_float_population[0]
#         parent2 = small_float_population[1]
        
#         offspring = crossover.crossover(parent1, parent2)
        
#         assert offspring is not None
#         assert len(offspring) == len(parent1)


# class TestMutatorOperators:
#     """Unit tests for mutator operators."""
    
#     @pytest.mark.unit
#     def test_arithmetic_mutator(self, small_float_population):
#         """Test arithmetic mutator operator."""
#         mutator = ArithmeticMutator(rate=0.1)
#         individual = small_float_population[0]
        
#         mutated = mutator.mutate(individual)
        
#         assert mutated is not None
#         assert len(mutated) == len(individual)
    
#     @pytest.mark.unit
#     def test_gaussian_mutator(self, small_float_population):
#         """Test gaussian mutator operator."""
#         mutator = GaussianMutator(rate=0.1, sigma=0.1)
#         individual = small_float_population[0]
        
#         mutated = mutator.mutate(individual)
        
#         assert mutated is not None
#         assert len(mutated) == len(individual)
    
#     @pytest.mark.unit
#     def test_invert_mutator(self, small_int_population):
#         """Test invert mutator operator."""
#         mutator = InvertMutator(rate=0.1)
#         individual = small_int_population[0]
        
#         mutated = mutator.mutate(individual)
        
#         assert mutated is not None
#         assert len(mutated) == len(individual)
    
#     @pytest.mark.unit
#     def test_scramble_mutator(self, small_int_population):
#         """Test scramble mutator operator."""
#         mutator = ScrambleMutator(rate=0.1)
#         individual = small_int_population[0]
        
#         mutated = mutator.mutate(individual)
        
#         assert mutated is not None
#         assert len(mutated) == len(individual)
    
#     @pytest.mark.unit
#     def test_swap_mutator(self, small_int_population):
#         """Test swap mutator operator."""
#         mutator = SwapMutator(rate=0.1)
#         individual = small_int_population[0]
        
#         mutated = mutator.mutate(individual)
        
#         assert mutated is not None
#         assert len(mutated) == len(individual)
    
#     @pytest.mark.unit
#     def test_uniform_mutator(self, small_int_population):
#         """Test uniform mutator operator."""
#         mutator = UniformMutator(rate=0.1)
#         individual = small_int_population[0]
        
#         mutated = mutator.mutate(individual)
        
#         assert mutated is not None
#         assert len(mutated) == len(individual)


# class TestSelectorOperators:
#     """Unit tests for selector operators."""
    
#     @pytest.mark.unit
#     def test_tournament_selector(self, small_int_population):
#         """Test tournament selector."""
#         selector = TournamentSelector(tournament_size=3)
        
#         # Create a simple population with scores
#         population = []
#         for i, individual in enumerate(small_int_population):
#             # Create a phenotype with score
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i)])  # Score based on index
#             population.append(phenotype)
        
#         selected = selector.select(population, rd.Objective.single(rd.Optimize.maximize), 2)
        
#         assert len(selected) == 2
#         assert all(isinstance(x, rd.Phenotype) for x in selected)
    
#     @pytest.mark.unit
#     def test_roulette_selector(self, small_int_population):
#         """Test roulette selector."""
#         selector = RouletteSelector()
        
#         # Create a simple population with scores
#         population = []
#         for i, individual in enumerate(small_int_population):
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i + 1)])  # Positive scores
#             population.append(phenotype)
        
#         selected = selector.select(population, rd.Objective.single(rd.Optimize.maximize), 2)
        
#         assert len(selected) == 2
#         assert all(isinstance(x, rd.Phenotype) for x in selected)
    
#     @pytest.mark.unit
#     def test_rank_selector(self, small_int_population):
#         """Test rank selector."""
#         selector = RankSelector()
        
#         # Create a simple population with scores
#         population = []
#         for i, individual in enumerate(small_int_population):
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i)])
#             population.append(phenotype)
        
#         selected = selector.select(population, rd.Objective.single(rd.Optimize.maximize), 2)
        
#         assert len(selected) == 2
#         assert all(isinstance(x, rd.Phenotype) for x in selected)
    
#     @pytest.mark.unit
#     def test_elite_selector(self, small_int_population):
#         """Test elite selector."""
#         selector = EliteSelector()
        
#         # Create a simple population with scores
#         population = []
#         for i, individual in enumerate(small_int_population):
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i)])
#             population.append(phenotype)
        
#         selected = selector.select(population, rd.Objective.single(rd.Optimize.maximize), 2)
        
#         assert len(selected) == 2
#         assert all(isinstance(x, rd.Phenotype) for x in selected)
    
#     @pytest.mark.unit
#     def test_boltzmann_selector(self, small_int_population):
#         """Test boltzmann selector."""
#         selector = BoltzmannSelector(temperature=4.0)
        
#         # Create a simple population with scores
#         population = []
#         for i, individual in enumerate(small_int_population):
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i)])
#             population.append(phenotype)
        
#         selected = selector.select(population, rd.Objective.single(rd.Optimize.maximize), 2)
        
#         assert len(selected) == 2
#         assert all(isinstance(x, rd.Phenotype) for x in selected)
    
#     @pytest.mark.unit
#     def test_stochastic_universal_sampling_selector(self, small_int_population):
#         """Test stochastic universal sampling selector."""
#         selector = StochasticUniversalSamplingSelector()
        
#         # Create a simple population with scores
#         population = []
#         for i, individual in enumerate(small_int_population):
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i + 1)])  # Positive scores
#             population.append(phenotype)
        
#         selected = selector.select(population, rd.Objective.single(rd.Optimize.maximize), 2)
        
#         assert len(selected) == 2
#         assert all(isinstance(x, rd.Phenotype) for x in selected)


# class TestGraphOperators:
#     """Unit tests for graph-specific operators."""
    
#     @pytest.mark.unit
#     def test_graph_crossover(self, graph_codec_simple):
#         """Test graph crossover operator."""
#         crossover = GraphCrossover(rate=0.5, connection_rate=0.5)
        
#         # Create two graph genotypes
#         genotype1 = graph_codec_simple.encode()
#         genotype2 = graph_codec_simple.encode()
        
#         offspring = crossover.crossover(genotype1, genotype2)
        
#         assert offspring is not None
#         assert len(offspring) == len(genotype1)
    
#     @pytest.mark.unit
#     def test_graph_mutator(self, graph_codec_simple):
#         """Test graph mutator operator."""
#         mutator = GraphMutator(rate=0.1, connection_rate=0.1)
        
#         # Create a graph genotype
#         genotype = graph_codec_simple.encode()
        
#         mutated = mutator.mutate(genotype)
        
#         assert mutated is not None
#         assert len(mutated) == len(genotype)
    
#     @pytest.mark.unit
#     def test_operation_mutator(self, graph_codec_simple):
#         """Test operation mutator operator."""
#         mutator = OperationMutator(rate=0.1, replace_rate=0.05)
        
#         # Create a graph genotype
#         genotype = graph_codec_simple.encode()
        
#         mutated = mutator.mutate(genotype)
        
#         assert mutated is not None
#         assert len(mutated) == len(genotype)


# class TestTreeOperators:
#     """Unit tests for tree-specific operators."""
    
#     @pytest.mark.unit
#     def test_tree_crossover(self, tree_codec_simple):
#         """Test tree crossover operator."""
#         crossover = TreeCrossover(rate=0.5)
        
#         # Create two tree genotypes
#         genotype1 = tree_codec_simple.encode()
#         genotype2 = tree_codec_simple.encode()
        
#         offspring = crossover.crossover(genotype1, genotype2)
        
#         assert offspring is not None
#         assert len(offspring) == len(genotype1)
    
#     @pytest.mark.unit
#     def test_hoist_mutator(self, tree_codec_simple):
#         """Test hoist mutator operator."""
#         mutator = HoistMutator(rate=0.1)
        
#         # Create a tree genotype
#         genotype = tree_codec_simple.encode()
        
#         mutated = mutator.mutate(genotype)
        
#         assert mutated is not None
#         assert len(mutated) == len(genotype)
    
#     @pytest.mark.unit
#     def test_subtree_mutator(self, tree_codec_simple):
#         """Test subtree mutator operator."""
#         mutator = SubtreeMutator(rate=0.1)
        
#         # Create a tree genotype
#         genotype = tree_codec_simple.encode()
        
#         mutated = mutator.mutate(genotype)
        
#         assert mutated is not None
#         assert len(mutated) == len(genotype)


# class TestOperatorEdgeCases:
#     """Unit tests for operator edge cases and error handling."""
    
#     @pytest.mark.unit
#     def test_crossover_empty_population(self):
#         """Test crossover handles empty population gracefully."""
#         crossover = UniformCrossover(rate=0.5)
        
#         with pytest.raises(ValueError):
#             crossover.crossover([], [])
    
#     @pytest.mark.unit
#     def test_mutator_empty_individual(self):
#         """Test mutator handles empty individual gracefully."""
#         mutator = ArithmeticMutator(rate=0.1)
        
#         with pytest.raises(ValueError):
#             mutator.mutate([])
    
#     @pytest.mark.unit
#     def test_selector_empty_population(self):
#         """Test selector handles empty population gracefully."""
#         selector = TournamentSelector(tournament_size=3)
        
#         with pytest.raises(ValueError):
#             selector.select([], rd.Objective.single(rd.Optimize.maximize), 1)
    
#     @pytest.mark.unit
#     def test_selector_invalid_selection_size(self):
#         """Test selector handles invalid selection size gracefully."""
#         selector = TournamentSelector(tournament_size=3)
        
#         # Create a simple population
#         population = []
#         for i in range(5):
#             individual = rd.Chromosome.int(length=3, value_range=(0, 10))
#             phenotype = rd.Phenotype(individual, generation=0)
#             phenotype.set_score([float(i)])
#             population.append(phenotype)
        
#         with pytest.raises(ValueError):
#             selector.select(population, rd.Objective.single(rd.Optimize.maximize), 10)  # More than population size
    
#     @pytest.mark.unit
#     def test_operator_parameter_validation(self):
#         """Test operator parameter validation."""
#         # Test invalid rates
#         with pytest.raises(ValueError):
#             UniformCrossover(rate=1.5)  # Rate > 1
        
#         with pytest.raises(ValueError):
#             ArithmeticMutator(rate=-0.1)  # Negative rate
        
#         with pytest.raises(ValueError):
#             TournamentSelector(tournament_size=0)  # Invalid tournament size 