from .op import Op
from .graph import Graph
from .tree import Tree

__all__ = ["Op", "Graph", "Tree"]

    # MAX_INDEX = 500
    # MIN_SCORE = 0.01

    # def get_dataset():
    #     inputs = [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]]
    #     answers = [[0.0], [0.0], [1.0], [1.0]]
    #     return rd.DataSet(inputs, answers)

    # engine = rd.GeneticEngine(
    #     problem=rd.Regression(
    #         dataset=get_dataset(),
    #         loss=rd.Loss.MSE,
    #         codec=rd.GraphCodec.directed(
    #             input_size=2,
    #             output_size=1,
    #             values=[
    #                 (rd.NodeType.Input, [rd.Op.var(0), rd.Op.var(1)]),
    #                 (rd.NodeType.Edge, [rd.Op.weight(), rd.Op.identity()]),
    #                 (rd.NodeType.Vertex, rd.ops.all_ops()),
    #                 (rd.NodeType.Output, [rd.Op.sigmoid()]),
    #             ]
    #         )
    #     ),
    #     minimizing=True,
    #     alters=[
    #         rd.GraphCrossover(0.5, 0.5),
    #         rd.OperationMutator(0.05, 0.05),
    #         rd.GraphMutator(0.06, 0.01).allow_recurrent(False)
    #     ]
    # )

    # result = engine.run(lambda ctx: (
    #     print(f"[ {ctx.index} ]: {ctx.score.as_f32()}"),
    #     ctx.index == MAX_INDEX or ctx.score.as_f32() < MIN_SCORE
    # ))