import radiate as rd
import numpy as np
import pytest


@pytest.mark.integration
def test_engine_int_minimization(random_seed):
    num_genes = 5

    engine = (
        rd.Engine.int(num_genes, init_range=(0, 10))
        .fitness(lambda x: sum(x))
        .minimizing()
    )

    result = engine.run(rd.ScoreLimit(0), rd.GenerationsLimit(500))

    assert result.value() == [0 for _ in range(num_genes)]
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MIN


@pytest.mark.integration
def test_engine_int_vector_nparray(random_seed):
    num_genes = 5

    def fit(x: np.ndarray) -> float:
        assert isinstance(x, np.ndarray)
        assert np.all(x >= -10) and np.all(x <= 50)
        assert x.dtype == np.int16
        return np.sum(x)

    result = (
        rd.Engine.int(
            num_genes,
            init_range=(0, 10),
            bounds=(-10, 50),
            use_numpy=True,
            dtype=rd.Int16,
        )
        .fitness(fit)
        .minimizing()
        .limit(rd.Limit.score(0), rd.Limit.generations(500))
    ).run()

    assert np.array_equal(result.value(), np.array([0 for _ in range(num_genes)]))
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MIN


@pytest.mark.integration
def test_engine_int_matrix_nparray(random_seed):
    rows, cols = 3, 4

    def fitness_func(x: np.ndarray) -> float:
        assert isinstance(x, np.ndarray)
        x = x.reshape((rows, cols))
        assert x.shape == (rows, cols)
        assert np.all(x >= -5) and np.all(x <= 20)
        assert x.dtype == np.int64
        return float(np.sum(x))

    engine = (
        rd.Engine.int(rows * cols, init_range=(0, 10), bounds=(-5, 20), use_numpy=True)
        .fitness(fitness_func)
        .minimizing()
    )

    result = engine.run(rd.ScoreLimit(0), rd.GenerationsLimit(500))

    assert np.array_equal(result.value(), np.zeros(rows * cols, dtype=np.int64))
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MIN


@pytest.mark.integration
def test_engine_int_jagged_matrix(random_seed):
    shape = [2, 3, 4]

    def fit(x: list[list[int]]) -> float:
        assert isinstance(x, list)
        assert all(isinstance(row, list) for row in x)
        assert len(x) == len(shape)
        for i, row in enumerate(x):
            assert len(row) == shape[i]
            for gene in row:
                assert isinstance(gene, int)
                assert -5 <= gene <= 20
        return sum(sum(row) for row in x)

    # Create a jagged matrix codec - right now (1/23/26) using numpy doesn't support non-square shapes
    codec = rd.IntCodec(shape=shape, init_range=(0, 10), bounds=(-5, 20))
    engine = rd.Engine(codec).fitness(fit).minimizing()
    result = engine.run(rd.Limit.score(0), rd.Limit.generations(500))

    assert result.value() == [[0 for _ in range(n)] for n in shape]
    assert result.score() == [0]
    assert result.index() <= 500
    assert len(result.population()) == len(result.ecosystem().population())
    assert len(result.ecosystem().species()) == 0
    assert result.objective() == rd.MIN


@pytest.mark.unit
def test_create_int_scalar_engine_from_genes():
    """Test creating a float engine from genes."""

    def scalar_fit(val: int) -> int:
        return val**2

    gene = rd.Gene.int(init_range=(-5, 5))
    engine = (
        rd.Engine.int(genes=gene).fitness(scalar_fit).limit(rd.Limit.generations(5))
    )

    result = engine.run()

    assert result is not None
    assert type(result.value()) is int


@pytest.mark.unit
def test_create_int_vector_engine_from_genes():
    """Test creating a float engine from genes."""

    def list_fit(val: list[int]) -> int:
        return sum(x**2 for x in val)

    def np_vector_fit(val: np.ndarray) -> float:
        assert isinstance(val, np.ndarray) and val.dtype == np.int64
        return float(np.sum(val**2))

    sequence = [
        rd.Gene.int(init_range=(-5, 5)),
        rd.Gene.int(init_range=(-50, 50)),
        rd.Gene.int(init_range=(10, 50)),
    ]

    result = (
        rd.Engine.int(genes=sequence).fitness(list_fit).limit(rd.Limit.generations(5))
    ).run()

    assert result is not None
    assert type(result.value()) is list
    assert len(result.value()) == 3

    for phenotype in result.population():
        for chromosome in phenotype.genotype():
            assert len(chromosome) == 3

            one = chromosome[0]
            two = chromosome[1]
            three = chromosome[2]

            assert one.allele() >= -5 and one.allele() <= 5
            assert two.allele() >= -50 and two.allele() <= 50
            assert three.allele() >= 10 and three.allele() <= 50

    engine = (
        rd.Engine.int(genes=sequence, use_numpy=True, dtype=rd.Int64)
        .fitness(np_vector_fit)
        .limit(rd.Limit.generations(5))
    )

    result = engine.run()

    assert result is not None
    assert type(result.value()) is np.ndarray
    assert result.value().shape == (3,)


@pytest.mark.unit
def test_create_int_engine_from_chromosomes():
    """Test creating a int engine from chromosomes."""

    def vector_fit(val: list[int]) -> float:
        return sum(x**2 for x in val)

    chromosome = rd.Chromosome.int(5, init_range=(-5, 5))
    engine = (
        rd.Engine.int(chromosomes=chromosome)
        .fitness(vector_fit)
        .limit(rd.Limit.generations(5))
    )

    result = engine.run()

    assert result is not None
    assert type(result.value()) is list
    assert len(result.value()) == 5

    def matrix_fit(val: list[list[int]]) -> int:
        return sum(sum(x**2 for x in row) for row in val)

    sequence = [
        rd.Chromosome.int(3, init_range=(-5, 5)),
        rd.Chromosome.int(3, init_range=(-50, 50)),
        rd.Chromosome.int(3, init_range=(10, 50)),
    ]

    engine = (
        rd.Engine.int(chromosomes=sequence)
        .fitness(matrix_fit)
        .limit(rd.Limit.generations(5))
    )

    result = engine.run()
    population = result.population()

    assert result is not None
    assert type(result.value()) is list
    assert len(result.value()) == 3

    for phenotype in population:
        genotype = phenotype.genotype()

        assert len(genotype) == 3

        one = genotype[0]
        two = genotype[1]
        three = genotype[2]

        assert len(one) == 3
        assert len(two) == 3
        assert len(three) == 3

        for gene in one:
            assert gene.allele() >= -5 and gene.allele() <= 5
        for gene in two:
            assert gene.allele() >= -50 and gene.allele() <= 50
        for gene in three:
            assert gene.allele() >= 10 and gene.allele() <= 50
