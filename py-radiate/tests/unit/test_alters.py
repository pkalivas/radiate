import pytest
import radiate as rd


class TestCustomMutators:
    """Tests for custom mutator base class and implementations."""

    @pytest.mark.unit
    def test_custom_mutator_inheritance(self):
        """Test that custom mutators properly inherit from Mutator base class."""

        class TestMutator(rd.Mutator):
            def mutate(self, chromosome: rd.Chromosome) -> rd.Chromosome:
                for i in range(len(chromosome)):
                    gene = chromosome.view(i)
                    chromosome[i] = gene.new_instance(gene.allele() * 2)
                return chromosome

        mutator = TestMutator()
        assert isinstance(mutator, rd.Mutator)
        assert hasattr(mutator, "mutate")
        assert callable(mutator.mutate)

    @pytest.mark.unit
    def test_custom_mutator_functionality(self):
        """Test that custom mutators perform the expected mutations."""

        class TestMutator(rd.Mutator):
            def mutate(self, chromosome: rd.Chromosome) -> rd.Chromosome:
                for gene in chromosome:
                    gene.apply(lambda allele: allele * 2)
                return chromosome

        original_chromosome = rd.Chromosome(rd.gene.float(i) for i in range(5))
        original_copy = original_chromosome.copy()

        mutator = TestMutator()
        mutated_chromosome = mutator.mutate(original_chromosome)

        assert mutated_chromosome == original_chromosome
        for i in range(len(original_chromosome)):
            assert mutated_chromosome[i].allele() == original_chromosome[i].allele()
            assert mutated_chromosome[i].allele() == original_copy[i].allele() * 2
