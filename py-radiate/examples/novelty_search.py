#!/usr/bin/env python3
"""
Robot Behavior Evolution with Novelty Search

This example demonstrates using novelty search to evolve diverse robot behaviors.
Instead of optimizing for a single objective (like distance traveled), we use
novelty search to find behaviors that are different from each other.

The robot has a simple 2D environment where it can move in different patterns.
Novelty search helps us discover various interesting behaviors like:
- Circular motion
- Zigzag patterns
- Spiral movements
- Random exploration
- etc.

This is a great example of how novelty search can find diverse solutions
that traditional fitness-based evolution might miss.
"""

import radiate as rd
import numpy as np
import matplotlib.pyplot as plt  # type: ignore
import math

# rd.random.set_seed(5522)  # For reproducibility
# np.random.seed(1220)  # For reproducibility


# rd.random.set_seed(552211)  # For reproducibility
# np.random.seed(1234)  # For reproducibility


class RobotBehavior:
    """Represents a robot's movement behavior in 2D space."""

    def __init__(self, movement_pattern: list[float]):
        """
        Initialize robot behavior.

        Args:
            movement_pattern: List of floats representing movement parameters
                           [step_size, angle_change, direction_bias, ...]
        """
        self.movement_pattern = movement_pattern
        self.trajectory = self._simulate_movement()

    def _simulate_movement(self, steps: int = 100) -> list[tuple[float, float]]:
        """Simulate the robot's movement and return trajectory."""
        x, y = 0.0, 0.0
        trajectory = [(x, y)]

        # Extract behavior parameters
        step_size = self.movement_pattern[0] if len(self.movement_pattern) > 0 else 1.0
        angle_change = (
            self.movement_pattern[1] if len(self.movement_pattern) > 1 else 0.1
        )
        direction_bias = (
            self.movement_pattern[2] if len(self.movement_pattern) > 2 else 0.0
        )
        noise_level = (
            self.movement_pattern[3] if len(self.movement_pattern) > 3 else 0.1
        )

        current_angle = 0.0

        for _ in range(steps):
            # Add noise and bias to angle
            angle_noise = np.random.normal(0, abs(noise_level))
            current_angle += angle_change + angle_noise + direction_bias

            # Calculate new position
            dx = step_size * math.cos(current_angle)
            dy = step_size * math.sin(current_angle)

            x += dx
            y += dy
            trajectory.append((x, y))

        return trajectory

    def get_behavior_descriptor(self) -> list[float]:
        """
        Extract behavior descriptor for novelty search.
        This describes the "shape" of the behavior.
        """
        if not self.trajectory:
            return [0.0] * 6

        # Calculate behavior characteristics
        x_coords = [p[0] for p in self.trajectory]
        y_coords = [p[1] for p in self.trajectory]

        # 1. Total distance traveled
        total_distance = sum(
            math.sqrt(
                (x_coords[i] - x_coords[i - 1]) ** 2
                + (y_coords[i] - y_coords[i - 1]) ** 2
            )
            for i in range(1, len(x_coords))
        )

        # 2. Final distance from start
        final_distance = math.sqrt(x_coords[-1] ** 2 + y_coords[-1] ** 2)

        # 3. Area covered (approximate)
        area = (max(x_coords) - min(x_coords)) * (max(y_coords) - min(y_coords))

        # 4. Directional bias (how much it moves in one direction)
        dx_total = sum(x_coords[i] - x_coords[i - 1] for i in range(1, len(x_coords)))
        dy_total = sum(y_coords[i] - y_coords[i - 1] for i in range(1, len(y_coords)))
        directional_bias = (
            math.sqrt(dx_total**2 + dy_total**2) / total_distance
            if total_distance > 0
            else 0
        )

        # 5. Path complexity (how "wiggly" the path is)
        path_complexity = len(self.trajectory) / (total_distance + 1)

        # 6. Return to start tendency
        return_to_start = 1.0 / (final_distance + 1)

        return [
            total_distance,
            final_distance,
            area,
            directional_bias,
            path_complexity,
            return_to_start,
        ]

    def visualize_behavior(self, title: str = "Robot Behavior"):
        """Visualize the robot's movement pattern."""
        if not self.trajectory:
            return

        x_coords = [p[0] for p in self.trajectory]
        y_coords = [p[1] for p in self.trajectory]

        plt.figure(figsize=(10, 8))
        plt.plot(x_coords, y_coords, "b-", linewidth=2, alpha=0.7, label="Path")
        plt.plot(x_coords[0], y_coords[0], "go", markersize=10, label="Start")
        plt.plot(x_coords[-1], y_coords[-1], "ro", markersize=10, label="End")

        # Add arrows to show direction
        for i in range(0, len(x_coords) - 1, 10):
            dx = x_coords[i + 1] - x_coords[i]
            dy = y_coords[i + 1] - y_coords[i]
            plt.arrow(
                x_coords[i],
                y_coords[i],
                dx,
                dy,
                head_width=0.2,
                head_length=0.3,
                fc="red",
                ec="red",
                alpha=0.5,
            )

        plt.xlabel("X Position")
        plt.ylabel("Y Position")
        plt.title(f"{title}\nBehavior: {self.movement_pattern[:4]}")
        plt.legend()
        plt.grid(True, alpha=0.3)
        plt.axis("equal")
        plt.show()


def run_novelty_search_evolution(generations: int = 200) -> rd.Generation:
    """Run novelty search to evolve diverse robot behaviors."""

    def behavior_descriptor(genome: list[float]) -> list[float]:
        behavior = RobotBehavior(genome)
        return behavior.get_behavior_descriptor()

    codec = rd.FloatCodec.vector(6, init_range=(-5.0, 5.0))

    # Create novelty search engine
    engine = rd.GeneticEngine(
        codec=codec,
        # Here we use a novelty search fitness function
        # This will not optimize for a single score,
        # but rather for diverse behaviors
        fitness_func=rd.NoveltySearch(
            descriptor=behavior_descriptor,
            distance=rd.CosineDistance(),
            k=15,  # Number of nearest neighbors
            threshold=0.6,  # Novelty threshold
            archive_size=1000,  # Maximum archive size
        ),
        survivor_selector=rd.TournamentSelector(3),
        offspring_selector=rd.BoltzmannSelector(4),
        alters=[
            rd.BlendCrossover(),
            rd.GaussianMutator(0.1),
        ],
    )

    return engine.run(rd.GenerationsLimit(generations), log=True)


def analyze_diverse_behaviors(result: rd.Generation, num_behaviors: int = 6):
    """Analyze and visualize diverse behaviors found by novelty search."""
    population = result.population()

    sorted_population = sorted(population, key=lambda x: x.score(), reverse=True)

    print("\n=== Novelty Search Results ===")
    print(f"Generations completed: {result.index()}")
    print(f"Population size: {len(population)}")
    print(f"Best novelty score: {sorted_population[0].score()[0]:.3f}")

    # Visualize top diverse behaviors
    plt.figure(figsize=(15, 10))

    for i in range(min(num_behaviors, len(sorted_population))):
        individual = sorted_population[i]
        genes = [g.allele() for c in individual.genotype() for g in c]
        behavior = RobotBehavior(genes)

        plt.subplot(2, 3, i + 1)
        x_coords = [p[0] for p in behavior.trajectory]
        y_coords = [p[1] for p in behavior.trajectory]

        plt.plot(x_coords, y_coords, linewidth=2, alpha=0.8)
        plt.plot(x_coords[0], y_coords[0], "go", markersize=8)
        plt.plot(x_coords[-1], y_coords[-1], "ro", markersize=8)

        plt.title(f"Behavior {i + 1}\nNovelty: {individual.score()[0]:.3f}")
        plt.grid(True, alpha=0.3)
        plt.axis("equal")

    plt.tight_layout()
    plt.show()

    # Print behavior characteristics
    print(f"\n=== Top {num_behaviors} Diverse Behaviors ===")
    for i in range(min(num_behaviors, len(sorted_population))):
        individual = sorted_population[i]
        genes = [g.allele() for c in individual.genotype() for g in c]

        behavior = RobotBehavior(genes)
        descriptor = behavior.get_behavior_descriptor()

        print(f"\nBehavior {i + 1} (Novelty: {individual.score()[0]:.3f}):")
        print(f"  Total Distance: {descriptor[0]:.2f}")
        print(f"  Final Distance: {descriptor[1]:.2f}")
        print(f"  Area Covered: {descriptor[2]:.2f}")
        print(f"  Directional Bias: {descriptor[3]:.2f}")
        print(f"  Path Complexity: {descriptor[4]:.2f}")


if __name__ == "__main__":
    print("=== Robot Behavior Evolution with Novelty Search ===")
    print("Evolving diverse robot movement behaviors...")

    # Run novelty search
    result = run_novelty_search_evolution(generations=550)

    # Analyze results
    analyze_diverse_behaviors(result, num_behaviors=6)

    # Show individual behaviors in detail
    print("\n=== Detailed Behavior Analysis ===")
    population = result.population()
    top_individuals = sorted(population, key=lambda x: x.score(), reverse=True)[:3]

    for i, individual in enumerate(top_individuals):
        genes = [g.allele() for c in individual.genotype() for g in c]

        behavior = RobotBehavior(genes)
        behavior.visualize_behavior(
            title=f"Top Behavior {i + 1} (Novelty: {individual.score()[0]:.3f})"
        )
