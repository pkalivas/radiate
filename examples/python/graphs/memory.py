import radiate as rd

data = rd.DataSet.simple_memory(30)
_, in_dim, out_dim = data.shape
features, targets = data.pair()

engine = (
    rd.Engine.graph(
        shape=(in_dim, out_dim),
        vertex=rd.Op.all_ops(),
        edge=rd.Op.weight(),
        output=rd.Op.sigmoid(),
        graph_type=rd.GraphType.GRU,
    )
    .regression(features, targets)
    .alter(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.1, 0.05),
        rd.Mutate.graph(0.05, 0.05, True),
    )
    .limit(rd.Limit.score(0.01), rd.Limit.generations(250))
)

result = engine.run(log=True)
print(result)
print(result.metrics.dashboard())
preds = result.value.eval(features)

for pred, target in zip(preds, targets):
    print(f"Target = {target[0]}, Predicted = {round(pred[0], 4)}")
