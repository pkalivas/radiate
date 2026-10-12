import plotly as pl
import radiate as rd

rd.random.seed(42)

x, y = rd.DataSet.pagie1().shuffle().pair()

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
    rd.Engine.graph(
        shape=(2, 1),
        vertex=vertex_ops,
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .regression(x, y, loss=rd.MSE)
    .select(rd.Select.boltzmann(temp=4.0))
    .alter(
        rd.Cross.graph(0.4, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.06, 0.06, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
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
