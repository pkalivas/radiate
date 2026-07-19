# --8<-- [start:eval]
import numpy as np
import radiate as rd

codec = rd.GraphCodec.directed(
    shape=(2, 1),
    vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
    edge=rd.Op.weight(),  # or [rd.Op.weight(), ...]
    output=rd.Op.linear(),  # or [rd.Op.linear(), ...]
)

graph = codec.decode(codec.encode())

inputs = np.array([1.0, 2.0])
outputs = graph.eval(inputs)  # list[float]

multi_inputs = np.array(
    [
        [1.0, 2.0],
        [3.0, 4.0],
    ]
)
multi_outputs = graph.eval(multi_inputs)  # list[list[float]]
# --8<-- [end:eval]

# --8<-- [start:variants]
import radiate as rd

# Every graph variant takes the same arguments, so define them once:
shape = (2, 1)
vertex = [rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()]
edge = [rd.Op.weight()]
output = [rd.Op.linear()]

codec = rd.GraphCodec.directed(shape, vertex=vertex, edge=edge, output=output)

# or recurrent graph
codec = rd.GraphCodec.recurrent(shape, vertex=vertex, edge=edge, output=output)

# or weighted directed graph
codec = rd.GraphCodec.weighted_directed(shape, vertex=vertex, edge=edge, output=output)

# or weighted recurrent graph
codec = rd.GraphCodec.weighted_recurrent(shape, vertex=vertex, edge=edge, output=output)

# or lstm graph
codec = rd.GraphCodec.lstm(shape, vertex=vertex, edge=edge, output=output)

# or gru graph
codec = rd.GraphCodec.gru(shape, vertex=vertex, edge=edge, output=output)

graph = codec.decode(codec.encode())

inputs = np.array([[1.0, 2.0]])
outputs = graph.eval(inputs)
# --8<-- [end:variants]

# --8<-- [start:encode_decode]
import radiate as rd

# Create a directed graph codec
codec = rd.GraphCodec.directed(
    shape=(2, 1),
    vertex=[rd.Op.add(), rd.Op.mul()],
    edge=rd.Op.weight(),
    output=rd.Op.linear(),
)

genotype = codec.encode()
graph = codec.decode(genotype)

# Create a recurrent graph codec
codec = rd.GraphCodec.recurrent(
    shape=(2, 1),
    vertex=[rd.Op.add(), rd.Op.mul()],
    edge=rd.Op.weight(),
    output=rd.Op.linear(),
)

genotype = codec.encode()
recurrent_graph = codec.decode(genotype)
# --8<-- [end:encode_decode]

# --8<-- [start:graph_mutator]
import radiate as rd

# Create a mutator that adds vertices and edges with a 10% chance for either
mutator = rd.Mutate.graph(
    vertex_rate=0.1, edge_rate=0.1, allow_recurrent=False
)  # Using the dsl syntax for mutators
# --8<-- [end:graph_mutator]

# --8<-- [start:graph_crossover]
import radiate as rd

crossover = rd.Cross.graph(0.1, 0.5)  # Using the dsl syntax for crossover operators
# --8<-- [end:graph_crossover]
