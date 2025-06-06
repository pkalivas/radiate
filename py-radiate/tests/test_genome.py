from __future__ import annotations

import radiate as rd


def test_float_gene_creation():
    gene = rd.Gene.float(value_range=(-10.0, 10.0))

    assert isinstance(gene.allele(), float)
    assert gene.allele() is not None
    assert gene.allele() >= -10.0 and gene.allele() <= 10.0


def test_int_gene_creation():
    gene = rd.Gene.int(value_range=(0, 10))

    assert isinstance(gene.allele(), int)
    assert gene.allele() is not None
    assert gene.allele() >= 0 and gene.allele() <= 10


def test_char_gene_creation():
    gene = rd.Gene.char(char_set={"a", "b", "c"})

    assert isinstance(gene.allele(), str)
    assert gene.allele() is not None
    assert gene.allele() in {"a", "b", "c"}


def test_bit_gene_creation():
    gene = rd.Gene.bit()

    assert isinstance(gene.allele(), int)
    assert gene.allele() is not None
    assert gene.allele() in {True, False}
