import pytest

import radiate as rd


@pytest.mark.integration
def test_engine_permutation_tsp(random_seed):
    """Test engine with permutation codec for TSP-like problem."""

    # Simple TSP-like fitness: minimize sum of adjacent differences
    def fit(x: list[int]) -> float:
        return sum(abs(x[i] - x[i - 1]) for i in range(1, len(x)))

    result = (
        rd.Engine.permutation([0, 1, 2, 3, 4])
        .fitness(fit)
        .minimizing()
        .size(50)
        .select(rd.Select.tournament(k=3), rd.Select.elite())
        .alter(rd.Cross.pmx(rate=0.7), rd.Mutate.inversion(rate=0.1))
        .limit(rd.Limit.score(5), rd.Limit.generations(100))
    ).run()

    assert result.index() <= 100
    assert len(set(result.value())) == 5
    assert all(0 <= x < 5 for x in result.value())


@pytest.mark.integration
def test_engine_permutation_returns_typed_object_to_fitness(random_seed):
    import math

    class MazeWaypoint:
        def __init__(self, x: float, y: float, name: str = ""):
            self.x = x
            self.y = y
            self.name = name

        def distance_to(self, other: "MazeWaypoint") -> float:
            return math.sqrt((self.x - other.x) ** 2 + (self.y - other.y) ** 2)

        def __repr__(self):
            return f"Waypoint({self.name}: {self.x:.1f}, {self.y:.1f})"

    points = [
        MazeWaypoint(0, 0, "A"),
        MazeWaypoint(1, 1, "B"),
        MazeWaypoint(2, 2, "C"),
        MazeWaypoint(3, 3, "D"),
        MazeWaypoint(4, 4, "E"),
        MazeWaypoint(5, 5, "F"),
        MazeWaypoint(6, 6, "G"),
        MazeWaypoint(7, 7, "H"),
        MazeWaypoint(8, 8, "I"),
    ]

    def fit(x: list[MazeWaypoint]) -> float:
        assert all(isinstance(val, MazeWaypoint) for val in x)
        return sum(x[i].distance_to(x[i - 1]) for i in range(1, len(x)))

    result = (
        rd.Engine.permutation(points)
        .fitness(fit)
        .minimizing()
        .size(50)
        .select(rd.Select.tournament(k=3), rd.Select.elite())
        .alter(rd.Cross.pmx(rate=0.7), rd.Mutate.inversion(rate=0.1))
        .limit(rd.Limit.score(0), rd.Limit.generations(100))
    ).run()

    assert result.index() <= 100
    assert set(result.value()) <= set(points)
