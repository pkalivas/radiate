#!/usr/bin/env python3
"""
Maze Solving with Permutation Codec

This example demonstrates using the PermutationCodec to solve a maze navigation problem.
We have a set of waypoints in a maze and need to find the optimal order to visit them.

This is pretty much a TSP (Traveling Salesman Problem) variant, where the start point is fixed
and we need to find the shortest path through a set of waypoints.
"""

import math

import matplotlib.pyplot as plt  # type: ignore
import radiate as rd
from matplotlib.markers import MarkerStyle  # type: ignore

START_POINT = (0, 0)
GENERATIONS = 250


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


def calculate_path_length(permutation: list[MazeWaypoint]) -> float:
    """Calculate the total path length for a given permutation."""
    if not permutation:
        return float("inf")

    total_distance = 0.0

    # Distance from start to first waypoint
    first_waypoint = permutation[0]
    total_distance += math.sqrt(
        (START_POINT[0] - first_waypoint.x) ** 2
        + (START_POINT[1] - first_waypoint.y) ** 2
    )

    # Distance between waypoints
    for i in range(len(permutation) - 1):
        wp1 = permutation[i]
        wp2 = permutation[i + 1]
        total_distance += wp1.distance_to(wp2)

    return total_distance


@rd.on_stop
def visualize_path(event: rd.EngineEvent):
    """Visualize the path through the maze."""

    path_length = calculate_path_length(event.value())

    _, ax = plt.subplots(figsize=(10, 8))

    x_coords = [wp.x for wp in waypoints]
    y_coords = [wp.y for wp in waypoints]
    ax.scatter(x_coords, y_coords, c="blue", s=100, alpha=0.6, label="Waypoints")

    ax.scatter(
        START_POINT[0],
        START_POINT[1],
        c="green",
        s=150,
        marker=MarkerStyle("o"),
        label="Start",
    )

    path_x = [START_POINT[0]] + [wp.x for wp in event.value()]
    path_y = [START_POINT[1]] + [wp.y for wp in event.value()]

    ax.plot(
        path_x,
        path_y,
        "r-",
        linewidth=2,
        alpha=0.8,
        label=f"Path (Length: {path_length:.2f})",
    )
    ax.scatter(path_x[1:], path_y[1:], c="red", s=80, alpha=0.8)

    for i, wp in enumerate(waypoints):
        ax.annotate(f"{i}", (wp.x, wp.y), xytext=(5, 5), textcoords="offset points")

    for i, wp in enumerate(event.value()):
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
    ax.legend()
    ax.grid(True, alpha=0.3)
    ax.set_aspect("equal")

    plt.tight_layout()
    plt.show()


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


engine = (
    rd.Engine.permutation(waypoints)
    .fitness(calculate_path_length)
    .minimizing()
    .subscribe(visualize_path)
    .alters(
        # PartiallyMappedCrossover and SwapMutator are common for TSP-like problems
        # where we want to maintain the permutation structure. ie., we don't want to
        # create duplicates or invalid permutations - we want to keep all waypoints and just
        # change their order around during crossover/mutation. There are a few other operators
        # that would also fit this type of problem, such as InversionMutator,
        # etc.
        rd.Cross.pmx(),  # Partially Mapped Crossover
        rd.Mutate.swap(),  # Swap Mutation
    )
    .limit(rd.Limit.generations(GENERATIONS))
)

result = engine.run(log=True)

print(result)

print("\nBest solution found:")
print(f"  Path: Start → {' → '.join([wp.name for wp in result.value()])} → End")
