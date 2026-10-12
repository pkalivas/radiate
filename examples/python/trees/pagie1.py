import plotly as pl
import radiate as rd

rd.random.seed(42)

data = rd.DataSet.pagie1()
x, y = data.shuffle().pair()
_, in_dim, out_dim = data.shape

vertex_ops = [
    rd.Op.abs(),
    rd.Op.log(),
    rd.Op.sigmoid(),
    rd.Op.tanh(),
    rd.Op.sin(),
    rd.Op.mul(),
    rd.Op.add(),
    rd.Op.div(),
]

engine = (
    rd.Engine.tree(
        shape=(in_dim, out_dim),
        vertex=vertex_ops,
        root=rd.Op.sum(),
        leaf=[rd.Op.var(0), rd.Op.var(1), rd.Op.erc()],
    )
    .regression(x, y, loss=rd.MSE)
    .select(rd.Select.boltzmann(temp=4.0))
    .diversity(rd.Dist.tree(), target=5)
    .alter(
        rd.Cross.tree(0.5),
        rd.Mutate.op(0.01, 0.08),
        rd.Mutate.subtree(0.1),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.seconds(10))
)


result = engine.run(log=True)

print(result)
print(result.metrics().dashboard())

x, y = rd.DataSet.pagie1().pair()
preds = result.value().eval(x)


fig = pl.graph_objects.Figure()
fig.add_trace(pl.graph_objects.Scatter(y=y.flatten(), mode="markers", name="True"))
fig.add_trace(
    pl.graph_objects.Scatter(y=preds.flatten(), mode="markers", name="Predicted")
)
fig.show(renderer="browser")
