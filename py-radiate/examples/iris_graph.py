import numpy as np  # type: ignore
import pandas as pd  # type: ignore
import radiate as rd
import requests  # type: ignore
from io import StringIO

rd.random.seed(500)

url = "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/iris.data"
response = requests.get(url, verify=False)
data = StringIO(response.text)

columns = ["sepal_length", "sepal_width", "petal_length", "petal_width", "species"]
table = pd.read_csv(data, header=None, names=columns)

# Normalize the data (only numeric columns)
numeric_columns = ["sepal_length", "sepal_width", "petal_length", "petal_width"]
species_column = [
    "species_Iris-setosa",
    "species_Iris-versicolor",
    "species_Iris-virginica",
]
for column in numeric_columns:
    table[column] = (table[column] - table[column].mean()) / table[column].std()


# One-hot encode the species column
table = pd.get_dummies(table, columns=["species"])

# Shuffle the dataframe
table = table.sample(frac=1, random_state=42).reset_index(drop=True)

# Split into training and testing data
cutoff = int(len(table) * 0.8)
training = table.iloc[:cutoff].copy()
testing = table.iloc[cutoff:].copy()

# Split features and targets
training_features = training[numeric_columns].copy()
training_target = training[species_column].copy()
testing_features = testing[numeric_columns].copy()
testing_target = testing[species_column].copy()

print(
    f"Training data shape: {training_features.shape}, Testing data shape: {testing_features.shape}"
)
print(
    f"Training target shape: {training_target.shape}, Testing target shape: {testing_target.shape}"
)

codec = rd.GraphCodec.directed(
    shape=(4, 4),
    vertex=rd.Op.all_ops(),
    edge=rd.Op.weight(),
    output=rd.Op.sigmoid(),
)

engine = rd.GeneticEngine(
    codec=codec,
    fitness_func=rd.Regression(
        training_features.values.tolist(), training_target.values.tolist()
    ),
    offspring_selector=rd.BoltzmannSelector(4),
    objective="min",
    alters=[
        rd.GraphCrossover(0.5, 0.5),
        rd.OperationMutator(0.02, 0.05),
        rd.GraphMutator(0.008, 0.002),
    ],
)

result = engine.run([rd.ScoreLimit(0.01), rd.SecondsLimit(5)], log=True)

eval_result = result.value().eval(testing_features.values.tolist())

accuracy = np.mean(
    np.argmax(eval_result, axis=1) == np.argmax(testing_target.values, axis=1)
)
print(f"Accuracy: {accuracy:.2f}")
