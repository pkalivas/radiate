# #!/usr/bin/env python3
# """
# Neural Network Evolution Performance Test
# Tests the impact of NumPy on genetic algorithm performance
# """

# import os
# import sys
# import time
# import numpy as np
# from typing import List, Tuple



# project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../py-radiate"))
# sys.path.insert(0, project_root)

# print(f"Project root: {project_root}")

# import radiate as rd


# class NeuralNetwork:
#     """Simple neural network for testing NumPy performance"""
    
#     def __init__(self, input_size: int, hidden_size: int, output_size: int):
#         self.input_size = input_size
#         self.hidden_size = hidden_size
#         self.output_size = output_size
        
#         # Calculate parameter counts
#         self.w1_size = input_size * hidden_size
#         self.w2_size = hidden_size * output_size
#         self.b1_size = hidden_size
#         self.b2_size = output_size
        
#         self.total_params = self.w1_size + self.w2_size + self.b1_size + self.b2_size
        
#     def set_weights(self, weights):
#         """Set network weights from flat array"""
#         if len(weights) != self.total_params:
#             raise ValueError(f"Expected {self.total_params} weights, got {len(weights)}")
        
#         idx = 0
        
#         # Extract weights and biases
#         self.w1 = np.array(weights[idx:idx + self.w1_size]).reshape(self.input_size, self.hidden_size)
#         idx += self.w1_size
        
#         self.w2 = np.array(weights[idx:idx + self.w2_size]).reshape(self.hidden_size, self.output_size)
#         idx += self.w2_size
        
#         self.b1 = np.array(weights[idx:idx + self.b1_size])
#         idx += self.b1_size
        
#         self.b2 = np.array(weights[idx:idx + self.b2_size])
        
#     def set_weights_list(self, weights):
#         """Set network weights from flat array as lists (for non-NumPy operations)"""
#         if len(weights) != self.total_params:
#             raise ValueError(f"Expected {self.total_params} weights, got {len(weights)}")
        
#         idx = 0
        
#         # Extract weights and biases as lists
#         w1_flat = weights[idx:idx + self.w1_size]
#         self.w1_list = []
#         for i in range(self.input_size):
#             row = []
#             for h in range(self.hidden_size):
#                 row.append(w1_flat[i * self.hidden_size + h])
#             self.w1_list.append(row)
#         idx += self.w1_size
        
#         w2_flat = weights[idx:idx + self.w2_size]
#         self.w2_list = []
#         for h in range(self.hidden_size):
#             row = []
#             for o in range(self.output_size):
#                 row.append(w2_flat[h * self.output_size + o])
#             self.w2_list.append(row)
#         idx += self.w2_size
        
#         self.b1_list = weights[idx:idx + self.b1_size]
#         idx += self.b1_size
        
#         self.b2_list = weights[idx:idx + self.b2_size]
    
#     def forward_without_numpy(self, input_data: List[float]) -> List[float]:
#         """Forward pass without NumPy (pure Python)"""
#         # Hidden layer
#         hidden = [0.0] * self.hidden_size
#         for h in range(self.hidden_size):
#             for i in range(self.input_size):
#                 hidden[h] += input_data[i] * self.w1_list[i][h]
#             hidden[h] += self.b1_list[h]
#             hidden[h] = np.tanh(hidden[h])
        
#         # Output layer
#         output = [0.0] * self.output_size
#         for o in range(self.output_size):
#             for h in range(self.hidden_size):
#                 output[o] += hidden[h] * self.w2_list[h][o]
#             output[o] += self.b2_list[o]
#             output[o] = np.tanh(output[o])
        
#         return output
    
#     def forward_with_numpy(self, input_data: np.ndarray) -> np.ndarray:
#         """Forward pass with NumPy"""
#         # Hidden layer
#         hidden = np.tanh(np.dot(input_data, self.w1) + self.b1)
        
#         # Output layer
#         output = np.tanh(np.dot(hidden, self.w2) + self.b2)
        
#         return output

# def create_xor_data() -> List[Tuple[List[float], List[float]]]:
#     """Create XOR training data"""
#     return [
#         ([0.0, 0.0], [0.0]),
#         ([0.0, 1.0], [1.0]),
#         ([1.0, 0.0], [1.0]),
#         ([1.0, 1.0], [0.0]),
#     ]

# def create_classification_data(n_samples: int = 100) -> List[Tuple[List[float], List[float]]]:
#     """Create classification training data"""
#     data = []
    
#     for _ in range(n_samples):
#         x = np.random.uniform(-1, 1)
#         y = np.random.uniform(-1, 1)
        
#         input_data = [x, y]
#         if x > 0 and y > 0:
#             output = [1.0, 0.0, 0.0]  # Class 0
#         elif x < 0 and y < 0:
#             output = [0.0, 1.0, 0.0]  # Class 1
#         else:
#             output = [0.0, 0.0, 1.0]  # Class 2
        
#         data.append((input_data, output))
    
#     return data

# def fitness_without_numpy(weights, shapes) -> float:
#     """Fitness function without NumPy"""
#     # Convert NumPy array to list if needed
#     if hasattr(weights, 'tolist'):
#         weights = weights.tolist()

#     input_size, hidden_size, output_size = shapes
#     network = NeuralNetwork(input_size, hidden_size, output_size)
#     network.set_weights_list(weights)
    
#     training_data = create_xor_data()
#     total_error = 0.0
    
#     for input_data, target in training_data:
#         output = network.forward_without_numpy(input_data)
#         error = (output[0] - target[0]) ** 2
#         total_error += error
    
#     return -total_error  # Negative because we want to minimize error

# def fitness_with_numpy(weights, shapes) -> float:
#     """Fitness function with NumPy"""
#     # Convert NumPy array to list if needed
#     if hasattr(weights, 'tolist'):
#         weights = weights.tolist()
    
#     input_size, hidden_size, output_size = shapes
#     network = NeuralNetwork(input_size, hidden_size, output_size)
#     network.set_weights(weights)
    
#     training_data = create_xor_data()
#     total_error = 0.0
    
#     for input_data, target in training_data:
#         input_array = np.array(input_data)
#         output = network.forward_with_numpy(input_array)
#         error = (output[0] - target[0]) ** 2
#         total_error += error
    
#     return -total_error

# def complex_fitness_without_numpy(weights, shapes) -> float:
#     """Complex fitness function without NumPy"""
#     try:
#         # # Convert NumPy array to list if needed
#         # if hasattr(weights, 'tolist'):
#         #     weights = weights.tolist()
        
#         input_size, hidden_size, output_size = shapes
#         network = NeuralNetwork(input_size, hidden_size, output_size)
#         network.set_weights_list(weights)
        
#         training_data = create_classification_data(100)
#         total_error = 0.0
        
#         for input_data, target in training_data:
#             output = network.forward_without_numpy(input_data)
            
#             # Calculate cross-entropy loss
#             loss = 0.0
#             for i in range(len(output)):
#                 predicted = (output[i] + 1.0) / 2.0  # Convert from [-1,1] to [0,1]
#                 predicted = max(0.001, min(0.999, predicted))  # Clamp to avoid log(0)
#                 loss -= target[i] * np.log(predicted) + (1.0 - target[i]) * np.log(1.0 - predicted)
            
#             total_error += loss
        
#         # Add L2 regularization
#         l2_reg = 0.01
#         weight_penalty = sum(w * w for w in weights) * l2_reg
    
#         return -(total_error + weight_penalty)

#     except Exception as e:
#         print(f"Error in complex_fitness_without_numpy: {weights}")
#         raise

# def complex_fitness_with_numpy(weights, shapes) -> float:
#     """Complex fitness function with NumPy"""
#     # Convert NumPy array to list if needed
#     # if hasattr(weights, 'tolist'):
#     #     print("Converting weights to list")
#     #     weights = weights.tolist()
    
#     input_size, hidden_size, output_size = shapes
#     network = NeuralNetwork(input_size, hidden_size, output_size)
#     network.set_weights(weights)
    
#     training_data = create_classification_data(100)
#     total_error = 0.0

#     inputs = [np.array(input_data) for input_data, _ in training_data]
#     targets = [np.array(target) for _, target in training_data]
    
#     for input_array, target in zip(inputs, targets):
#         output = network.forward_with_numpy(input_array)
        
#         # Calculate cross-entropy loss with NumPy
#         predicted = (output + 1.0) / 2.0  # Convert from [-1,1] to [0,1]
#         predicted = np.clip(predicted, 0.001, 0.999)  # Clamp to avoid log(0)
#         loss = -np.sum(target * np.log(predicted) + (1.0 - target) * np.log(1.0 - predicted))
        
#         total_error += loss
    
#     # Add L2 regularization with NumPy
#     l2_reg = 0.01
#     weight_penalty = np.sum(np.array(weights) ** 2) * l2_reg
    
#     return -(total_error + weight_penalty)

# def benchmark_fitness_functions():
#     """Benchmark different fitness function implementations"""

    

#     # Test parameters
#     cplx_input_size, cplx_hidden_size, cplx_output_size = 2, 8, 3
#     cplx_network = NeuralNetwork(2, 8, 3)
#     cplx_weights = cplx_network.total_params

#     xor_input_size, xor_hidden_size, xor_output_size = 2, 4, 1
#     xor_network = NeuralNetwork(xor_input_size, xor_hidden_size, xor_output_size)
    
#     # Generate test weights
#     test_weights = np.random.uniform(-1, 1, cplx_weights).tolist()
#     xor_weights = xor_network.total_params
#     xor_weights = np.random.uniform(-1, 1, xor_weights).tolist()

#     # Test functions
#     test_functions = [
#         ("Simple XOR (No NumPy)", lambda w, s: fitness_without_numpy(w, s), False),
#         ("Simple XOR (With NumPy)", lambda w, s: fitness_with_numpy(w, s), False),
#         ("Complex Classification (No NumPy)", lambda w, s: complex_fitness_without_numpy(w, s), True),
#         ("Complex Classification (With NumPy)", lambda w, s: complex_fitness_with_numpy(w, s), True),
#     ]
    
#     print("Fitness Function Performance Benchmark")
#     print("=" * 60)
#     print(f"Network: {cplx_input_size} -> {cplx_hidden_size} -> {cplx_output_size}")
#     print(f"Total parameters: {cplx_weights}")
#     print()
    
#     results = []

#     for name, func, is_complex in test_functions:
#         # Warm up
#         weights = test_weights if is_complex else xor_weights
#         shape = (cplx_input_size, cplx_hidden_size, cplx_output_size) if is_complex else (xor_input_size, xor_hidden_size, xor_output_size)
#         for _ in range(10):
#             func(weights, shape)

#         # Benchmark
#         start_time = time.time()
#         for _ in range(1000):
#             result = func(weights, shape)
#         end_time = time.time()
        
#         total_time = end_time - start_time
#         avg_time = total_time / 1000 * 1000  # Convert to milliseconds
        
#         results.append((name, total_time, avg_time, result))
        
#         print(f"{name:35} | {total_time:.4f}s | {avg_time:.2f}ms per call | Fitness: {result:.4f}")
    
#     print()
#     print("Performance Analysis:")
#     print("-" * 30)
    
#     # Compare NumPy vs No NumPy for simple case
#     simple_numpy_time = results[1][1]
#     simple_no_numpy_time = results[0][1]
#     simple_speedup = simple_no_numpy_time / simple_numpy_time
    
#     # Compare NumPy vs No NumPy for complex case
#     complex_numpy_time = results[3][1]
#     complex_no_numpy_time = results[2][1]
#     complex_speedup = complex_no_numpy_time / complex_numpy_time
    
#     print(f"Simple XOR - NumPy speedup: {simple_speedup:.2f}x")
#     print(f"Complex Classification - NumPy speedup: {complex_speedup:.2f}x")
    
#     return results

# def test_genetic_algorithm_performance():
#     """Test NumPy impact in actual genetic algorithm"""
    
#     print("\n" + "=" * 60)
#     print("Genetic Algorithm Performance Test")
#     print("=" * 60)
    
#     # Test parameters
#     input_size, hidden_size, output_size = 2, 8, 3
#     network = NeuralNetwork(input_size, hidden_size, output_size)
#     total_weights = network.total_params

#     print(f"total_weights: {total_weights}")
    
#     population_size = 100
#     generations = 20
    
#     print(f"Network: {input_size} -> {hidden_size} -> {output_size}")
#     print(f"Parameters: {total_weights}")
#     print(f"Population: {population_size}")
#     print(f"Generations: {generations}")
#     print()
    
#     # Test without NumPy
#     print("Running GA without NumPy...")
#     start_time = time.time()
    
#     codec = rd.FloatCodec.vector(total_weights, value_range=(-2.0, 2.0), bound_range=(-2.0, 2.0), use_numpy=False)
#     engine = rd.GeneticEngine(
#         codec=codec,
#         fitness_func=lambda w: complex_fitness_with_numpy(w, (input_size, hidden_size, output_size)),
#         population_size=population_size,
#         survivor_selector=rd.TournamentSelector(k=3),
#         offspring_selector=rd.RouletteSelector(),
#         alters=[
#             rd.UniformCrossover(rate=0.7),
#             rd.GaussianMutator(rate=0.1),
#         ],
#         objectives="max"
#     ) 
    
#     result = engine.run(rd.GenerationsLimit(generations))
#     time_without_numpy = time.time() - start_time
    
#     print(f"Without NumPy: {time_without_numpy:.2f}s, Best: {result.score()}")

#     codec = rd.FloatCodec.vector(total_weights, value_range=(-2.0, 2.0), bound_range=(-2.0, 2.0), use_numpy=True)
#     # Test with NumPy
#     print("Running GA with NumPy...")
#     start_time = time.time()
    
#     engine = rd.GeneticEngine(
#         codec=codec,
#         fitness_func=lambda w: complex_fitness_with_numpy(w, (input_size, hidden_size, output_size)),
#         population_size=population_size,
#         survivor_selector=rd.TournamentSelector(k=3),
#         offspring_selector=rd.RouletteSelector(),
#         alters=[
#             rd.UniformCrossover(rate=0.7),
#             rd.GaussianMutator(rate=0.1),
#         ],
#         objectives="max"
#     )
    
#     result = engine.run(rd.GenerationsLimit(generations))
#     time_with_numpy = time.time() - start_time
    
#     print(f"With NumPy: {time_with_numpy:.2f}s, Best: {result.score()}")
    
#     # Calculate speedup
#     speedup = time_without_numpy / time_with_numpy
#     print(f"\nOverall speedup: {speedup:.2f}x")
    
#     return time_without_numpy, time_with_numpy, speedup

# def test_scaling():
#     """Test performance scaling with different problem sizes"""
    
#     print("\n" + "=" * 60)
#     print("Scaling Analysis")
#     print("=" * 60)
    
#     network_sizes = [

#         (2, 8, 3),    # Medium
#         (2, 16, 3),   # Large
#         # (2, 32, 3),   # Very large
#         # (2, 64, 3),   # Extra large
#         # (2, 128, 3),  # Huge
#         # (2, 256, 3),  # Massive
#         # (2, 512, 3),  # Gigantic
#         # (2, 1024, 3), # Colossal
#         # (2, 2048, 3), # Monstrous
#         # (2, 4096, 3)  # Behemoth
#     ]
    
#     population_size = 50
#     generations = 10
    
#     results = []
    
#     for input_size, hidden_size, output_size in network_sizes:
#         shape = (input_size, hidden_size, output_size)
#         network = NeuralNetwork(input_size, hidden_size, output_size)
#         total_weights = network.total_params
        
#         print(f"\nTesting network: {input_size} -> {hidden_size} -> {output_size}")
#         print(f"Parameters: {total_weights}")
        
    
#         # Test without NumPy
#         start_time = time.time()
#         codec = rd.FloatCodec.vector(total_weights, value_range=(-2.0, 2.0), bound_range=(-2.0, 2.0), use_numpy=False)
#         engine = rd.GeneticEngine(
#             codec=codec,
#             fitness_func=lambda w: complex_fitness_with_numpy(w, shape),
#             population_size=population_size,
#             survivor_selector=rd.TournamentSelector(k=3),
#             offspring_selector=rd.RouletteSelector(),
#             alters=[rd.UniformCrossover(rate=0.7), rd.GaussianMutator(rate=0.1)],
#             objectives="max"
#         )
#         result = engine.run(rd.GenerationsLimit(generations))
#         time_without = time.time() - start_time
        
#         # Test with NumPy
#         codec = rd.FloatCodec.vector(total_weights, value_range=(-2.0, 2.0), use_numpy=True)
#         start_time = time.time()
#         engine = rd.GeneticEngine(
#             codec=codec,
#             fitness_func=lambda x: complex_fitness_with_numpy(x, shape),
#             population_size=population_size,
#             survivor_selector=rd.TournamentSelector(k=3),
#             offspring_selector=rd.RouletteSelector(),
#             alters=[rd.UniformCrossover(rate=0.7), rd.GaussianMutator(rate=0.1)],
#             objectives="max"
#         )
#         result = engine.run(rd.GenerationsLimit(generations))
#         time_with = time.time() - start_time
        
#         speedup = time_without / time_with
#         results.append((total_weights, time_without, time_with, speedup))
        
#         print(f"  Without NumPy: {time_without:.2f}s")
#         print(f"  With NumPy: {time_with:.2f}s")
#         print(f"  Speedup: {speedup:.2f}x")
    
#     print("\nScaling Summary:")
#     print("-" * 20)
#     for params, time_without, time_with, speedup in results:
#         print(f"{params:4} params: {speedup:.2f}x speedup")

# if __name__ == "__main__":
#     print("Neural Network Evolution - NumPy Performance Test")
#     print("=" * 70)
    
#     # Run benchmarks
#     # benchmark_functions = benchmark_fitness_functions()
#     # ga_performance = test_genetic_algorithm_performance()
#     test_scaling()

#     print("\n" + "=" * 70)