#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


codec = rd.FloatCodec([
    rd.FloatGene(1.0),
    rd.FloatGene(2.0),
    rd.FloatGene(init_range=(-10.0, 10.0))
])

print(codec.encode()[0][1:])


engine = rd.GeneticEngine(
    codec=rd.FloatCodec.vector(3, (-10.0, 10.0)),
    fitness_func=lambda x: sum(x),
    objectives="min",
)

print(engine.__dict__())

print(rd.gene.float(0.1))