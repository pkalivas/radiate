# --8<-- [start:graph_regression]
import radiate as rd

rd.random.seed(518)


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

# Create a simple dataset of 20 points where the input is between -1 and 1
# and the output is the result of the compute function above
input = -1.0
for _ in range(-10, 10):
    input += 0.1
    inputs.append([input])
    answers.append([compute(input)])

# Create a simple GraphCodec which takes one input and produces one output
# The graph will have vertex nodes that can add, subtract, or multiply
# The edges will have a weight operation and the output will be linear
codec = rd.GraphCodec.directed(
    shape=(1, 1),
    vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
    edge=rd.Op.weight(),
    output=rd.Op.linear(),
)

# All we have to do to create a regression problem is provide the features & targets for our
# dataset. Optionally, we can provide a loss function as well - the default is mean squared error (MSE).
# The last argument is whether to use batch evaluation or not - the default is False. This has minimal impact on performance.
loss = rd.MSE  # Options are: rd.MSE, rd.MAE, rd.XEnt (CrossEntropy), rd.Diff.
fitness_func = rd.Regression(inputs, answers, loss=loss, batch=False)

engine = (
    rd.Engine(codec)
    .fitness(fitness_func)
    .minimizing()  # We want to minimize the loss
    .limit(
        rd.Limit.score(0.001), rd.Limit.generations(100)
    )  # Stop when we reach a loss of 0.001 or after 100 generations
    .alters(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(
            0.1, 0.1, allow_recurrent=False
        ),  # True if evolving recurrent graphs is allowed
    )
)

# Run the genetic engine with a score (error) limit of 0.001 or a maximum of 100 generations
result = engine.run(log=True)
# --8<-- [end:graph_regression]

# --8<-- [start:dataframe_regression]
import radiate as rd
import polars as pl

rd.random.seed(518)

# Just a simple dataset with 2 features and a target that is a function of those features.
df = pl.DataFrame(
    {
        "feature_one": [i for i in range(20)],
        "feature_two": [i**2 for i in range(20)],
        "target": [4.0 * i**3 - 3.0 * i**2 + i for i in range(20)],
    }
)

# Build a regression that evolves Graphs with two input features and one output
# (notice that the shape of the Graphs is equal to the number of features and targets in the DataFrame).
# The loss function is mean absolute error (MAE) in this case, but that's only to show flexibility.
engine = (
    rd.Engine.graph(
        shape=(
            2,
            1,
        ),  # <- notice how we have two features now, so the input shape is (2, 1)
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .regression(
        df,
        target_cols=["target"],
        feature_cols=["feature_one", "feature_two"],
        loss=rd.MAE,
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(100))
    .alters(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1),
    )
)

result = engine.run(log=True)
# --8<-- [end:dataframe_regression]
