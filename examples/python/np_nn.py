"""
This example shows how to use Radiate with NumPy arrays.

It implements a simple feedforward neural network with 3 layers (input, hidden, output) to fit a regression problem.
The network weights are evolved using a float codec.
"""

from pathlib import Path

import numpy as np  # type: ignore
import radiate as rd

rd.random.seed(123)

ROOT = Path(__file__).parent.parent
WRITE_DIR = ROOT / "data" / "scratch"
READ_DIR = ROOT / "data" / "scratch" / "chckpnt_50.json"


def compute(x: float) -> float:
    return 4.0 * x**3 - 3.0 * x**2 + x


inputs = []
answers = []

x = -1.0
for _ in range(20):
    x += 0.1
    inputs.append([x])
    answers.append([compute(x)])

X = np.array(inputs, dtype=np.float32)  # (N, 1)
Y = np.array(answers, dtype=np.float32)  # (N, 1)

# Add bias term: (N, 2) = [x, 1]
Xb = np.concatenate([X, np.ones((X.shape[0], 1), dtype=np.float32)], axis=1)


def fit(weights: list[np.ndarray]) -> float:
    # Decode weights
    W1 = weights[0].reshape((8, 2))
    W2 = weights[1].reshape((8, 8))
    W3 = weights[2].reshape((1, 8))

    # Forward pass
    # Xb: (N,2)
    h1 = Xb @ W1.T  # (N,2) @ (2,8) => (N,8)
    h1 = np.maximum(0, h1)  # ReLU activation

    h2 = h1 @ W2  # (N,8) @ (8,8) => (N,8)
    h2 = np.tanh(h2)  # tanh activation

    yhat = h2 @ W3.T  # (N,8) @ (8,1) => (N,1)

    # MSE
    return float(np.mean((yhat - Y) ** 2, dtype=np.float32))


@rd.on_stop
def metrics_dashboard(event: rd.EngineEvent):
    print(event.metrics().dashboard())


@rd.on_epoch
def my_logger(event: rd.EngineEvent):
    print(f"Epoch {event.index()}: Best score = {event.score():.6f}")


engine = (
    rd.Engine.float(
        # Create an engine that evolves genomes with 3 chromosomes, one for each layer's weights, 1 with 16 genes, 1 with 64 genes, and 1 with 8 genes
        shape=[16, 64, 8],
        # Each gene is initialized randomly in the range [-1, 1]
        init_range=(-1.0, 1.0),
        # Genes aren't allowed to go outside the range [-3, 3] during evolution
        bounds=(-3.0, 3.0),
        # Decode radiate's backend (rust) chromosomes into numpy arrays for the fitness function
        use_numpy=True,
        # Use 32-bit floats in radiate's backend (rust side) - note the numpy arrays will also be float32, so we avoid unnecessary up/down casting
        dtype=rd.Float32,
    )
    .fitness(fit)
    .minimizing()
    .subscribe(metrics_dashboard, my_logger)
    .select(rd.Select.boltzmann(temp=4.0))
    .alters(rd.Cross.blend(0.7, 0.4), rd.Mutate.gaussian(0.1))
    .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
)

# import threading
# import time


# class BackgroundWorker:
#     def __init__(self, target_function, *args, **kwargs):
#         """
#         Accepts any function and its arguments to run in the background.
#         """
#         self.target = target_function
#         self.args = args
#         self.kwargs = kwargs
#         self.thread = None

#     def __enter__(self):
#         # Create and start the thread immediately
#         self.thread = threading.Thread(
#             target=self.target,
#             args=self.args,
#             kwargs=self.kwargs,
#             daemon=True,  # Ensures the thread dies if the main program crashes
#         )
#         self.thread.start()
#         return self  # Control moves instantly to the code inside the 'with' block

#     def __exit__(self, exc_type, exc_val, exc_tb):
#         # Wait for the background task to finish before leaving the block
#         if self.thread and self.thread.is_alive():
#             print("\n[Front] Main tasks done. Waiting for background task to finish...")
#             self.thread.join()
#         print("[Front] Background worker context closed.")


# # 1. Define your slow background task
# def heavy_download(file_name, duration):
#     print(f"[Back] Starting download of {file_name}...")
#     for i in range(1, duration + 1):
#         time.sleep(1)
#         print(f"[Back] Downloading... {i}/{duration}s")
#     print(f"[Back] {file_name} successfully downloaded!")


# # 2. Run the front-end and back-end simultaneously
# print("[Front] Program started.")

# with BackgroundWorker(heavy_download, "dataset.csv", duration=4):
#     # This code executes immediately while the download runs in parallel
#     print("[Front] User interface is responsive! Type data or process inputs here.")
#     for step in ["A", "B", "C"]:
#         time.sleep(0.8)
#         print(f"[Front] Processing frontend step {step}...")

# print("[Front] Program complete.")

# # print(engine.run(ui=True))

# # one = next(engine)
# # print(one)
# # two = next(engine)
# # print(two)
# # three = next(engine)
# # print(three)

# # control = engine.control()

# # four = next(engine)
# # print(four)

# # control.pause()

# # five = next(engine)
# # print(five)


# # .load_checkpoint(
# #     READ_DIR, ignore_not_found=True
# # )  # Load from a previous checkpoint if it exists

# # checkpoint = (50, WRITE_DIR, "pkl")

# # for metric in metrics.values_by_tag(rd.Tag.DERIVED):
# #     print(metric)


# # .load_checkpoint(
# #         READ_DIR, ignore_not_found=True
# #     )  # Load from a previous checkpoint if it exists
