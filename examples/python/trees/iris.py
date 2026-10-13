#!/usr/bin/env python3
import radiate as rd

rd.random.seed(500)

data = rd.DataSet.iris()
train, test = data.split(0.75)
train_features, train_targets = train.standardize().pair()
test_features, test_targets = test.standardize().pair()
_, in_dim, out_dim = data.shape


engine = (
    rd.Engine.tree(
        shape=(in_dim, out_dim),
        root=rd.Op.sigmoid(),
        vertex=[
            rd.Op.sigmoid(),
            rd.Op.add(),
            rd.Op.mul(),
            rd.Op.sub(),
            rd.Op.dot(),
        ],
        leaf=rd.Op.erc(),
    )
    .regression(train_features, train_targets, loss=rd.MSE)
    .alter(
        rd.Cross.tree(0.7),
        rd.Mutate.op(0.01, 0.08),
        rd.Mutate.subtree(0.05, max_depth=5),
    )
    .limit(rd.Limit.score(0.1), rd.Limit.seconds(5))
)


result = engine.run(log=True)


print(result.metrics().dashboard())
print(result)
print(
    rd.accuracy(
        result.value(),
        features=train_features,
        targets=train_targets,
        name="TRAIN_ACCURACY",
        loss=rd.MSE,
    )
)
print(
    rd.accuracy(
        result.value(),
        features=test_features,
        targets=test_targets,
        name="TEST_ACCURACY",
        loss=rd.MSE,
    )
)
