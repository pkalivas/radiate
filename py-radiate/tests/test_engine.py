from __future__ import annotations

import radiate as rd


def test_simple_min_sum_engine():
    num_genes = 5  # Number of genes in the genome
    engine = rd.GeneticEngine(
        codec=rd.IntCodec.vector(num_genes, value_range=(0, 10)),
        fitness_func=lambda x: sum(x),
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert result.value() == [0 for _ in range(num_genes)]
    assert result.score() == [0]
    assert result.index() < 500
    