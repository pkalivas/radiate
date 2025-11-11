import pytest
import radiate as rd
from radiate.genome import GeneType


class TestAnyCodec:
    @pytest.mark.unit
    def test_any_codec_creation(self):
        class MyGene(rd.AnyGene):
            def __init__(self):
                self.one = 1
                self.two = "two"
                self.three = 3.0
                self.four = [4, 4, 4]

        codec = rd.AnyCodec(MyGene() for _ in range(2))

        genotype = codec.encode()

        assert genotype is not None
        assert genotype.gene_type() == GeneType.ANY
        assert len(genotype) == 1
        assert len(genotype[0]) == 2

        decoded = codec.decode(genotype)

        for d in decoded:
            assert isinstance(d, MyGene)
            assert d.one == 1
            assert d.two == "two"
            assert d.three == 3.0
            assert d.four == [4, 4, 4]

    @pytest.mark.unit
    def test_complex_any_codec(self):
        class ComplexGene(rd.AnyGene):
            def __init__(self):
                self.name = "ComplexGene"
                self.attributes = {
                    "attr1": 1,
                    "attr2": "two",
                    "attr3": 3.0,
                    "attr4": [4, 4, 4],
                }

        codec = rd.AnyCodec(ComplexGene() for _ in range(2))

        genotype = codec.encode()

        assert genotype is not None
        assert genotype.gene_type() == GeneType.ANY
        assert len(genotype) == 1
        assert len(genotype[0]) == 2

        decoded = codec.decode(genotype)

        for d in decoded:
            assert isinstance(d, ComplexGene)
            assert d.name == "ComplexGene"
            assert d.attributes == {
                "attr1": 1,
                "attr2": "two",
                "attr3": 3.0,
                "attr4": [4, 4, 4],
            }

    @pytest.mark.unit
    def test_ensure_fitness_fn_recieves_custom_gene_as_input(self):
        class CustomGene(rd.AnyGene):
            def __init__(self):
                self.value = 42

        def fitness_fn(genes: list[CustomGene]) -> float:
            assert all(isinstance(g, CustomGene) for g in genes)
            return sum(g.value for g in genes)

        codec = rd.AnyCodec(CustomGene() for _ in range(3))

        engine = rd.GeneticEngine(codec, fitness_fn)
        result = engine.run(rd.GenerationsLimit(1))

        assert isinstance(result.value(), list)
        assert all(isinstance(g, CustomGene) for g in result.value())

    @pytest.mark.unit
    def test_any_gene_with_list_inner_value(self):
        class ListGene(rd.AnyGene):
            def __init__(self):
                self.data = [i for i in range(10)]

        codec = rd.AnyCodec([ListGene(), ListGene()])
        genotype = codec.encode()
        decoded = codec.decode(genotype)

        assert len(decoded) == 2
        for gene in decoded:
            assert isinstance(gene, ListGene)
            assert gene.data == [i for i in range(10)]