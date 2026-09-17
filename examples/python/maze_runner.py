#!/usr/bin/env python3
"""
Maze Solving with Permutation Codec

This example demonstrates using the PermutationCodec to solve a maze navigation problem.
We have a set of waypoints in a maze and need to find the optimal order to visit them.

This is pretty much a TSP (Traveling Salesman Problem) variant, where the start point is fixed
and we need to find the shortest path through a set of waypoints.
"""

import math

import plotly.graph_objects as go

import radiate as rd

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

    fig = go.Figure()

    x_coords = [wp.x for wp in waypoints]
    y_coords = [wp.y for wp in waypoints]
    fig.add_trace(
        go.Scatter(
            x=x_coords,
            y=y_coords,
            mode="markers",
            marker={"color": "blue", "size": 14, "opacity": 0.6},
            name="Waypoints",
        )
    )

    fig.add_trace(
        go.Scatter(
            x=[START_POINT[0]],
            y=[START_POINT[1]],
            mode="markers",
            marker={"color": "green", "size": 18, "symbol": "circle"},
            name="Start",
        )
    )

    path_x = [START_POINT[0]] + [wp.x for wp in event.value()]
    path_y = [START_POINT[1]] + [wp.y for wp in event.value()]

    fig.add_trace(
        go.Scatter(
            x=path_x,
            y=path_y,
            mode="lines",
            line={"color": "red", "width": 2},
            opacity=0.8,
            name=f"Path (Length: {path_length:.2f})",
        )
    )
    fig.add_trace(
        go.Scatter(
            x=path_x[1:],
            y=path_y[1:],
            mode="markers",
            marker={"color": "red", "size": 11, "opacity": 0.8},
            showlegend=False,
        )
    )

    for i, wp in enumerate(waypoints):
        fig.add_annotation(
            x=wp.x, y=wp.y, text=str(i), showarrow=False, xshift=8, yshift=8
        )

    for i, wp in enumerate(event.value()):
        fig.add_annotation(
            x=wp.x,
            y=wp.y,
            text=f"<b>→{i + 1}</b>",
            showarrow=False,
            xshift=16,
            yshift=16,
            font={"color": "red"},
        )

    fig.update_layout(
        xaxis_title="X Coordinate",
        yaxis_title="Y Coordinate",
        yaxis={"scaleanchor": "x", "scaleratio": 1},
    )
    fig.show()


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
    .alter(
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
