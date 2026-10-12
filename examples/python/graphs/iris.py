#!/usr/bin/env python3
import radiate as rd

rd.random.seed(500)


train, test = rd.DataSet.iris().split(0.75)
train_features, train_targets = train.standardize().pair()
test_features, test_targets = test.standardize().pair()


engine = (
    rd.Engine.graph(
        shape=(4, 3),
        vertex=rd.Op.all_ops(),
        edge=rd.Op.weight(),
        output=rd.Op.sigmoid(),
    )
    .regression(train_features, train_targets, loss=rd.MSE)
    .alter(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.02, 0.05),
        rd.Mutate.graph(0.008, 0.002, False),
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
