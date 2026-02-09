import pytest
import radiate as rd


def points_to_phenotypes(points: list[list[float]]) -> list[rd.Phenotype]:
    """Convert a list of points to a list of Phenotypes."""
    phenotypes = []
    for point in points:
        min_point = min(point)
        max_point = max(point)
        genes = [rd.gene.float(g, init_range=(min_point, max_point)) for g in point]
        chromosome = rd.chromosome.float(genes=genes)
        genotype = rd.Genotype(chromosome)
        phenotype = rd.Phenotype(genotype, score=point)
        phenotypes.append(phenotype)
    return phenotypes


def front_max(r: float, count: int) -> list[rd.Phenotype]:
    """Generate a list of points on the Pareto front for a given radius and count."""
    import math

    return points_to_phenotypes(
        [r * math.sin(a), r * math.cos(a)]
        for a in (rd.random.float() * math.pi * 0.5 for _ in range(count))
    )


def front_min(r: float, count: int) -> list[rd.Phenotype]:
    """Generate a list of points on the Pareto front for a given radius and count."""
    import math

    return points_to_phenotypes(
        [r * math.sin(a), r * math.cos(a)]
        for a in (rd.random.float() * math.pi * 0.5 + math.pi for _ in range(count))
    )


@pytest.mark.unit
def test_add_to_front():
    """Test adding items to the front."""
    chromosome = rd.chromosome.int(length=3, init_range=(0, 10))
    genotype = rd.Genotype(chromosome)
    phenotype = rd.Phenotype(genotype)

    front = rd.Front(objectives=["min"], range=(0, 100))
    result = front.add([phenotype])

    assert isinstance(result, dict) or result is None
    assert "added" in result
    assert result["added"] == 1
    assert len(front) == 1


@pytest.mark.unit
def test_front_can_max():
    """Test that the front can be added to with points on the Pareto front."""
    front = rd.Front(objectives=["max", "max"], range=(0, 5000))

    rank0 = front_max(20.0, 50)
    rank1 = front_max(15.0, 50)
    rank2 = front_max(10.0, 50)
    rank3 = front_max(5.0, 50)
    rank4 = front_max(1.0, 25)

    add_result = front.add(rank0 + rank1 + rank2 + rank3 + rank4)
    first_front = front.fronts()[0]
    first_front_scores = set(tuple(member.score()) for member in first_front.values())
    rank0_scores = set(tuple(member.score()) for member in rank0)

    assert len(first_front) == add_result["added"]
    assert len(first_front) == len(rank0)
    assert first_front_scores == rank0_scores


@pytest.mark.unit
def test_front_can_min():
    """Test that the front can be added to with points on the Pareto front."""
    front = rd.Front(objectives=["min", "min"], range=(0, 5000))
    rank0 = front_min(20.0, 50)
    rank1 = front_min(15.0, 50)
    rank2 = front_min(10.0, 50)
    rank3 = front_min(5.0, 50)
    rank4 = front_min(1.0, 25)

    add_result = front.add(rank0 + rank1 + rank2 + rank3 + rank4)
    first_front = front.fronts()[0]
    first_front_scores = set(tuple(member.score()) for member in first_front.values())
    rank0_scores = set(tuple(member.score()) for member in rank0)

    assert len(first_front) == add_result["added"]
    assert len(first_front) == len(rank0)
    assert first_front_scores == rank0_scores


