#!/usr/bin/env python3
"""
Maze Solving with Permutation Codec

This example demonstrates using the PermutationCodec to solve a maze navigation problem.
We have a set of waypoints in a maze and need to find the optimal order to visit them.

This is pretty much a TSP (Traveling Salesman Problem) variant, where the start point is fixed
and we need to find the shortest path through a set of waypoints.
"""

import radiate as rd
import matplotlib.pyplot as plt  # type: ignore
import math


class MazeWaypoint:
    """Represents a waypoint in the maze."""

    def __init__(self, x: float, y: float, name: str = ""):
        self.x = x
        self.y = y
        self.name = name

    def distance_to(self, other: "MazeWaypoint") -> float:
        """Calculate Euclidean distance to another waypoint."""
        return math.sqrt((self.x - other.x) ** 2 + (self.y - other.y) ** 2)

    def __repr__(self):
        return f"Waypoint({self.name}: {self.x:.1f}, {self.y:.1f})"


class MazeSolver:
    """Maze solving problem using permutation codec."""

    def __init__(
        self, waypoints: list[MazeWaypoint], start_point: tuple[float, float] = (0, 0)
    ):
        self.waypoints = waypoints
        self.start_x, self.start_y = start_point
        self.codec = rd.PermutationCodec(self.waypoints)

    def calculate_path_length(self, permutation: list[MazeWaypoint]) -> float:
        """Calculate the total path length for a given permutation."""
        if not permutation:
            return float("inf")

        total_distance = 0.0

        # Distance from start to first waypoint
        first_waypoint = permutation[0]
        total_distance += math.sqrt(
            (self.start_x - first_waypoint.x) ** 2
            + (self.start_y - first_waypoint.y) ** 2
        )

        # Distance between waypoints
        for i in range(len(permutation) - 1):
            wp1 = permutation[i]
            wp2 = permutation[i + 1]
            total_distance += wp1.distance_to(wp2)

        return total_distance

    def visualize_path(self, permutation: list[int], title: str = "Maze Path"):
        """Visualize the path through the maze."""
        if hasattr(permutation, "tolist"):
            permutation = permutation.tolist()

        path_length = self.calculate_path_length(permutation)

        _, ax = plt.subplots(figsize=(10, 8))

        x_coords = [wp.x for wp in self.waypoints]
        y_coords = [wp.y for wp in self.waypoints]
        ax.scatter(x_coords, y_coords, c="blue", s=100, alpha=0.6, label="Waypoints")

        ax.scatter(
            self.start_x, self.start_y, c="green", s=150, marker="s", label="Start"
        )

        path_x = [self.start_x] + [wp.x for wp in permutation]
        path_y = [self.start_y] + [wp.y for wp in permutation]

        ax.plot(
            path_x,
            path_y,
            "r-",
            linewidth=2,
            alpha=0.8,
            label=f"Path (Length: {path_length:.2f})",
        )
        ax.scatter(path_x[1:], path_y[1:], c="red", s=80, alpha=0.8)

        for i, wp in enumerate(self.waypoints):
            ax.annotate(f"{i}", (wp.x, wp.y), xytext=(5, 5), textcoords="offset points")

        for i, wp in enumerate(permutation):
            ax.annotate(
                f"→{i + 1}",
                (wp.x, wp.y),
                xytext=(10, 10),
                textcoords="offset points",
                color="red",
                fontweight="bold",
            )

        ax.set_xlabel("X Coordinate")
        ax.set_ylabel("Y Coordinate")
        ax.set_title(title)
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_aspect("equal")

        plt.tight_layout()
        plt.show()


def run_maze_evolution(
    maze_solver: MazeSolver, generations: int = 100
) -> rd.Generation:
    engine = rd.GeneticEngine(
        codec=maze_solver.codec,
        fitness_func=maze_solver.calculate_path_length,
        survivor_selector=rd.TournamentSelector(3),
        offspring_selector=rd.BoltzmannSelector(3),
        objective="min",
        alters=[
            # PartiallyMappedCrossover and SwapMutator are common for TSP-like problems
            # where we want to maintain the permutation structure. ie., we don't want to
            # create duplicates or invalid permutations - we want to keep all waypoints and just
            # change their order around during crossover/mutation. There are a few other operators
            # that would also fit this type of problem, such as InversionMutator,
            # etc. See the For a little more color, check out the docs:
            # https://pkalivas.github.io/radiate/source/alterers/
            rd.PartiallyMappedCrossover(),
            rd.SwapMutator(),
        ],
    )

    return engine.run(rd.GenerationsLimit(generations), log=True)


if __name__ == "__main__":
    waypoints = [
        MazeWaypoint(2, 3, "A"),
        MazeWaypoint(5, 1, "B"),
        MazeWaypoint(8, 4, "C"),
        MazeWaypoint(1, 6, "D"),
        MazeWaypoint(7, 7, "E"),
        MazeWaypoint(4, 8, "F"),
        MazeWaypoint(6, 5, "G"),
        MazeWaypoint(3, 2, "H"),
        MazeWaypoint(9, 9, "I"),
        MazeWaypoint(0, 0, "J"),
        MazeWaypoint(2, 8, "K"),
        MazeWaypoint(5, 6, "L"),
        MazeWaypoint(8, 2, "M"),
        MazeWaypoint(1, 4, "N"),
        MazeWaypoint(7, 3, "O"),
    ]

    maze_solver = MazeSolver(waypoints, start_point=(0, 0))
    result = run_maze_evolution(maze_solver, generations=250)

    permutation = result.value()

    print("\nBest solution found:")
    print(f"  Generations completed: {result.index()}")
    print(f"  Path length: {result.score()}")
    print(f"  Path: Start → {' → '.join([wp.name for wp in permutation])} → End")
    print(f"  Permutation: {permutation}")

    maze_solver.visualize_path(
        permutation, title=f"Best Maze Path (Length: {result.score()})"
    )
