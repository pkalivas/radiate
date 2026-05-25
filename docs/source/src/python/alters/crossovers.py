# --8<-- [start:blend_crossover]
import radiate as rd

crossover = rd.BlendCrossover(rate=0.1, alpha=0.5)
crossover = rd.Cross.blend(rate=0.1, alpha=0.5)  # Using the Cross dsl syntax
# --8<-- [end:blend_crossover]

# --8<-- [start:intermediate_crossover]
import radiate as rd

crossover = rd.IntermediateCrossover(rate=0.1, alpha=0.5)
crossover = rd.Cross.intermediate(rate=0.1, alpha=0.5)  # Using the Cross dsl syntax
# --8<-- [end:intermediate_crossover]

# --8<-- [start:mean_crossover]
import radiate as rd

crossover = rd.MeanCrossover(rate=0.1)
crossover = rd.Cross.mean(rate=0.1)  # Using the Cross dsl syntax
# --8<-- [end:mean_crossover]

# --8<-- [start:multipoint_crossover]
import radiate as rd

crossover = rd.MultiPointCrossover(rate=0.1, num_points=2)
crossover = rd.Cross.multipoint(rate=0.1, num_points=2)  # Using the Cross dsl syntax
# --8<-- [end:multipoint_crossover]

# --8<-- [start:pmx_crossover]
import radiate as rd

crossover = rd.PartiallyMappedCrossover(rate=0.1)
crossover = rd.Cross.pmx(rate=0.1)  # Using the Cross dsl syntax
# --8<-- [end:pmx_crossover]

# --8<-- [start:edge_recombination_crossover]
import radiate as rd

crossover = rd.EdgeRecombinationCrossover(rate=0.1)
crossover = rd.Cross.edge_recombination(rate=0.1)  # Using the Cross dsl syntax
# --8<-- [end:edge_recombination_crossover]

# --8<-- [start:shuffle_crossover]
import radiate as rd

crossover = rd.ShuffleCrossover(rate=0.1)
crossover = rd.Cross.shuffle(rate=0.1)  # Using the Cross dsl syntax
# --8<-- [end:shuffle_crossover]

# --8<-- [start:sbx_crossover]
import radiate as rd

crossover = rd.SimulatedBinaryCrossover(rate=0.1, contiguity=0.5)
crossover = rd.Cross.sbx(rate=0.1, contiguity=0.5)  # Using the Cross dsl syntax
# --8<-- [end:sbx_crossover]

# --8<-- [start:uniform_crossover]
import radiate as rd

crossover = rd.UniformCrossover(rate=0.1)
crossover = rd.Cross.uniform(rate=0.1)  # Using the Cross dsl syntax
# --8<-- [end:uniform_crossover]
