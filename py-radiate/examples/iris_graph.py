#!/usr/bin/env python3
# /// script
# requires-python = ">=3.13"
# dependencies = [
#   "polars",
#   "requests",
# ]
# ///

import os
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import polars as pl
import radiate as rd

rd.random.seed(500)

url = "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/iris.data"

numeric_cols = [
    "sl",
    "sw",
    "pl",
    "pw",
]

df = pl.read_csv(
    url,
    has_header=False,
    new_columns=[
        "sl",
        "sw",
        "pl",
        "pw",
        "species",
    ],
)

# Filter out rows with missing or empty species values.
df = df.filter(pl.col("species").is_not_null() & (pl.col("species") != ""))

# Standardize numeric columns.
df = df.with_columns(
    [
        ((pl.col(col) - pl.col(col).mean()) / pl.col(col).std()).alias(col)
        for col in numeric_cols
    ]
)

# One-hot encode species.
df = df.to_dummies(columns=["species"])
species_cols = df[[c for c in df.columns if c.startswith("species_")]].columns

# Shuffle rows.
df = df.sample(fraction=1.0, shuffle=True, seed=42)

# Split into training and testing sets.
training = df.head(int(len(df) * 0.8))
testing = df.tail(int(len(df) * 0.2))

# Separate features and targets.
train_features = training[numeric_cols]
train_targets = training[species_cols]
test_features = testing[numeric_cols]
test_targets = testing[species_cols]


engine = (
    rd.Engine.graph(
        shape=(4, 3),
        vertex=rd.Op.all_ops(),
        edge=rd.Op.weight(),
        output=rd.Op.sigmoid(),
    )
    .regression(features=train_features, targets=train_targets, loss=rd.MSE)
    .alters(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.02, 0.05),
        rd.Mutate.graph(0.008, 0.002, False),
    )
    .limit(rd.Limit.score(0.01), rd.Limit.seconds(1))
)

result = engine.run(log=True)


print(result.metrics().dashboard())
print(result.value())
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


outputs = []
targs = test_targets.to_numpy().tolist()
evaled = result.value().eval(test_features.to_numpy().tolist())
for i, row in enumerate(test_features.to_numpy().tolist()):
    output = evaled[i]
    outputs.append(output)
    max_idx = output.index(max(output))
    target_max_idx = targs[i].index(max(targs[i]))

    print(f"Predicted Class: {max_idx}, Target Class: {target_max_idx}")
