# /// script
# requires-python = ">=3.13"
# dependencies = [
#   "torch",
#   "torchvision",
#   "numpy",
# ]
# ///

# import radiate using os.path to ensure it works when run from the examples directory
import os
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import radiate as rd
import torch  # type: ignore
from torch import nn
import torch.nn.functional as F  # type: ignore
import numpy as np

import torchvision.datasets as datasets
import torchvision.transforms as transforms

transform = transforms.Compose(
    [transforms.ToTensor(), transforms.Normalize((0.1307,), (0.3081,))]
)

train_dataset = datasets.MNIST(
    root="./data", train=True, download=True, transform=transform
)
test_dataset = datasets.MNIST(
    root="./data", train=False, download=True, transform=transform
)


rd.random.seed(42)
torch.manual_seed(42)
np.random.seed(42)
