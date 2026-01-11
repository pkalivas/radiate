
# Custom Genes

Although radiate provides a variety of built-in gene types, you can create your own custom genes by subclassing the `Gene` class. In general, its going to be best to model you're problem domain as closely as possible with the built-in genes, but sometimes you may need a specialized gene type. In rust, this is done by implementing the `Gene` trait for your custom struct, while in python you can subclass the `AnyGene` class.

`AnyGene` follows how [polars](https://docs.pola.rs) handles custom data types by using a generic `Any` type. This means that you can create genes that hold any type of data you want, as long as its a valid python object.

Simply inherit from `rd.AnyGene` and implement the necessary methods such as `__init__`, `__repr__`, and any other methods you may need for your specific use case.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    from datetime import datetime, timezone

    rd.random.seed(42)


    class ObjectGene(rd.AnyGene):
        def __init__(self):
            self.number = rd.random.int(min=0, max=10)
            self.date = datetime(2020, 1, 1, tzinfo=timezone.utc)

        def __repr__(self):
            return f"ObjectGene(number={self.number}, date={self.date})"

    def fitness_function(phenotypes: list[list[ObjectGene]]) -> list[float]:
        return [sum(gene.number for gene in individual) for individual in phenotypes]

    engine = rd.GeneticEngine(
        rd.AnyCodec(ObjectGene() for _ in range(10)),
        fitness_func=rd.BatchFitness(fitness_function),
        objective="min",
    )

    result = engine.run(rd.ScoreLimit(0), ui=True)

    for obj_gene in result.value():
        print(obj_gene)
    ```

`AnyGene` is compatible with the rest of the radiate system, meaning alters and other components can work with it seamlessly.

Valid alters for `AnyGene` include:

- `MultiPointCrossover`
- `UniformCrossover`
- `ShuffleCrossover`
- `MeanCrossover`
- `SwapMutator`
- `ScrambleMutator`
- `InversionMutator`
- `UniformMutator`
- `ArithmeticMutator`

Converting between rust and python custom genes behind the scenes is a bit more involved that other genes, so reach for built-in genes when possible for best performance.