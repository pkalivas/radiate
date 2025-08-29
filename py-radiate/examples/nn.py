#!/usr/bin/env python3
"""
PyTorch Neural Network Weight Evolution with Radiate FloatCode
"""

import radiate as rd
import torch  # type: ignore
import torch.nn.functional as F  # type: ignore
import numpy as np
# import matplotlib.pyplot as plt # type: ignore

rd.random.seed(42)
torch.manual_seed(42)
np.random.seed(42)


class PyTorchNeuralNetwork:
    """A PyTorch neural network with evolvable weights"""

    def __init__(self, input_size: int, hidden_size: int, output_size: int):
        """
        Initialize the neural network architecture.

        Args:
            input_size: Number of input features
            hidden_size: Number of hidden neurons
            output_size: Number of output neurons
        """
        self.input_size = input_size
        self.hidden_size = hidden_size
        self.output_size = output_size

        # Calculate total number of weights and biases
        self.num_weights_1 = input_size * hidden_size  # Input to hidden weights
        self.num_biases_1 = hidden_size  # Hidden layer biases
        self.num_weights_2 = hidden_size * output_size  # Hidden to output weights
        self.num_biases_2 = output_size  # Output layer biases

        self.total_params = (
            self.num_weights_1
            + self.num_biases_1
            + self.num_weights_2
            + self.num_biases_2
        )

        self.weights_1 = None
        self.biases_1 = None
        self.weights_2 = None
        self.biases_2 = None

        print("Neural Network Architecture:")
        print(f"  Input size: {input_size}")
        print(f"  Hidden size: {hidden_size}")
        print(f"  Output size: {output_size}")
        print(f"  Total parameters: {self.total_params}")
        print()

    def set_weights(self, weight_vector: list[float]) -> None:
        """
        Set the network weights from a flat vector.

        Args:
            weight_vector: Flat list of weights and biases
        """
        if len(weight_vector) != self.total_params:
            raise ValueError(
                f"Expected {self.total_params} weights, got {len(weight_vector)}"
            )

        # Extract weights and biases from the flat vector
        start_idx = 0

        # Input to hidden weights
        end_idx = start_idx + self.num_weights_1
        self.weights_1 = torch.tensor(
            weight_vector[start_idx:end_idx], dtype=torch.float32
        )
        self.weights_1 = self.weights_1.view(self.input_size, self.hidden_size)
        start_idx = end_idx

        # Hidden layer biases
        end_idx = start_idx + self.num_biases_1
        self.biases_1 = torch.tensor(
            weight_vector[start_idx:end_idx], dtype=torch.float32
        )
        start_idx = end_idx

        # Hidden to output weights
        end_idx = start_idx + self.num_weights_2
        self.weights_2 = torch.tensor(
            weight_vector[start_idx:end_idx], dtype=torch.float32
        )
        self.weights_2 = self.weights_2.view(self.hidden_size, self.output_size)
        start_idx = end_idx

        # Output layer biases
        end_idx = start_idx + self.num_biases_2
        self.biases_2 = torch.tensor(
            weight_vector[start_idx:end_idx], dtype=torch.float32
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass through the network.

        Args:
            x: Input tensor of shape (batch_size, input_size)

        Returns:
            Output tensor of shape (batch_size, output_size)
        """
        # Hidden layer with ReLU activation
        hidden = F.relu(torch.mm(x, self.weights_1) + self.biases_1)

        # Output layer (linear)
        output = torch.mm(hidden, self.weights_2) + self.biases_2

        return output

    def predict(self, x: torch.Tensor) -> torch.Tensor:
        """Make predictions with the network."""
        # Remove the eval() call since this class doesn't inherit from nn.Module
        with torch.no_grad():
            return self.forward(x)


class NeuralNetworkEvolver:
    """Evolves PyTorch neural network weights using Radiate"""

    def __init__(self, input_size: int, hidden_size: int, output_size: int):
        """
        Initialize the neural network evolver.

        Args:
            input_size: Number of input features
            hidden_size: Number of hidden neurons
            output_size: Number of output neurons
        """
        self.network = PyTorchNeuralNetwork(input_size, hidden_size, output_size)
        self.X = torch.tensor(
            [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]], dtype=torch.float32
        )

        self.y = torch.tensor([[0.0], [1.0], [1.0], [0.0]], dtype=torch.float32)

    def create_fitness_function(self):
        """Create fitness function that evaluates neural network performance"""

        def fitness_function(weight_vector: list[float]) -> float:
            """
            Evaluate the fitness of a weight vector.

            Args:
                weight_vector: Flat list of weights and biases

            Returns:
                Fitness score (lower is better for error)
            """
            self.network.set_weights(weight_vector)
            predictions = self.network.forward(self.X)
            mse = F.mse_loss(predictions, self.y)
            return mse.item()

        return fitness_function

    def create_engine(self) -> rd.GeneticEngine:
        """Create the genetic engine for evolving neural network weights"""
        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(
                length=self.network.total_params,
                init_range=(-2.0, 2.0),
                bounds=(-5.0, 5.0),
            ),
            fitness_func=self.create_fitness_function(),
        )

        engine.minimizing()
        engine.survivor_selector(rd.BoltzmannSelector(temp=2.3))
        engine.alters(
            [
                rd.BlendCrossover(0.7, 0.5),
                rd.IntermediateCrossover(0.6, 0.5),
            ]
        )

        return engine

    def run_evolution(self, max_generations: int = 500, target_error: float = 0.01):
        """Run the neural network weight evolution"""

        print("Starting Neural Network Weight Evolution")
        print("=" * 50)
        print(f"Target error: {target_error}")
        print(f"Max generations: {max_generations}")
        print()

        # Create and run the engine
        engine = self.create_engine()

        # Define stopping criteria
        limits = [
            rd.ScoreLimit(target_error),  # Stop when error is low enough
            rd.GenerationsLimit(max_generations),  # Stop after max generations
        ]

        # Run evolution with logging
        result = engine.run(limits, log=True)

        return result

    def evaluate_best_network(self, best_weights: list[float]):
        """Evaluate and display the best evolved network"""

        print("\n" + "=" * 50)
        print("BEST EVOLVED NEURAL NETWORK")
        print("=" * 50)

        # Set the best weights
        self.network.set_weights(best_weights)

        print("XOR Problem Results:")
        print("Input\t\tExpected\tPredicted\tError")
        print("-" * 50)

        total_error = 0.0
        for i in range(len(self.X)):
            input_data = self.X[i : i + 1]  # Add batch dimension
            target = self.y[i].item()

            # Get prediction
            prediction = self.network.predict(input_data)
            predicted = prediction.item()

            error = abs(target - predicted)
            total_error += error

            print(f"{self.X[i].tolist()}\t\t{target}\t\t{predicted:.4f}\t\t{error:.4f}")

        print("-" * 50)
        print(f"Average Error: {total_error / len(self.X):.4f}")
        print(f"Network successfully learned XOR: {total_error / len(self.X) < 0.1}")

        return self.network

    def visualize_weights(self, network: PyTorchNeuralNetwork):
        """Visualize the evolved network weights"""

        print("\n" + "=" * 30)
        print("EVOLVED WEIGHTS")
        print("=" * 30)

        print("Input to Hidden Weights:")
        print(network.weights_1.numpy())
        print()

        print("Hidden Biases:")
        print(network.biases_1.numpy())
        print()

        print("Hidden to Output Weights:")
        print(network.weights_2.numpy())
        print()

        print("Output Biases:")
        print(network.biases_2.numpy())
        print()


def main():
    """Main function to run the neural network weight evolution"""

    # Create the evolver with a 2-4-1 network (2 inputs, 4 hidden, 1 output)
    evolver = NeuralNetworkEvolver(input_size=2, hidden_size=4, output_size=1)

    # Run evolution
    result = evolver.run_evolution(max_generations=300, target_error=0.01)

    # Get the best individual (weight vector)
    best_weights = result.value()

    # Evaluate the best network
    best_network = evolver.evaluate_best_network(best_weights)

    # Visualize the weights
    evolver.visualize_weights(best_network)

    print("\nEvolution completed!")
    print(f"Final score: {result.score()}")
    print(f"Generations: {result.index()}")


if __name__ == "__main__":
    main()
