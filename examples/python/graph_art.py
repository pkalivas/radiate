#!/usr/bin/env python3
"""
Drawing with Graphs: Compositional Pattern-Producing Networks (CPPNs)

Instead of directly encoding pixels or polygons, we evolve a small
`Graph<Op<f32>>` -- a tiny mathematical function of a pixel's coordinates --
and query it at every pixel to produce an entire image. This "indirect
encoding" is the classic CPPN trick behind projects like Picbreeder: because
the genome is a compact function built from sin/cos/tanh/etc, small graphs
tend to produce organic, symmetric, repeating patterns rather than noise.

The CPPN is trained with plain regression (the same `.regression()`
convenience used by the graph_xor/tree examples) against a real target image,
so this is a nice direct comparison to `mona_lisa.py`'s approach to the same
problem: a function of (x, y) vs. a soup of polygons.

Requires: pillow, numpy  (`uv pip install pillow numpy`)
"""

import math
from pathlib import Path

import numpy as np  # type: ignore
import radiate as rd
from PIL import Image  # type: ignore

rd.random.seed(42)

RESOLUTION = 48  # the target is downscaled to RESOLUTION x RESOLUTION for training
RENDER_SIZE = 256  # the CPPN is a continuous function, so it upsamples for free
GENERATIONS = 1500

ROOT = Path(__file__).parent.parent
OUT = ROOT / "data" / "results" / "graph_art"
OUT.mkdir(parents=True, exist_ok=True)

target_img = Image.open(ROOT / "data" / "monalisa.png").convert("RGB")
target_img = target_img.resize((RESOLUTION, RESOLUTION))
target_arr = np.asarray(target_img, dtype=np.float32) / 255.0  # (H, W, 3)


def coords(size: int) -> list[list[float]]:
    """Normalized (x, y) in [-1, 1], their radius from center, and a bias term
    -- the classic CPPN input scheme. This lets the evolved function key off
    position, distance-from-center, or a fixed offset (via the bias) as it
    likes."""
    grid = []
    for row in range(size):
        y = (row / (size - 1)) * 2.0 - 1.0
        for col in range(size):
            x = (col / (size - 1)) * 2.0 - 1.0
            grid.append([x, y, math.sqrt(x * x + y * y), 1.0])
    return grid


inputs = coords(RESOLUTION)
answers = [
    target_arr[row, col].tolist()
    for row in range(RESOLUTION)
    for col in range(RESOLUTION)
]

codec = rd.GraphCodec.directed(
    shape=(4, 3),
    vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sin(), rd.Op.cos(), rd.Op.tanh()],
    edge=rd.Op.weight(),
    output=rd.Op.sigmoid(),  # keep RGB output squashed into [0, 1]
)

engine = (
    rd.Engine(codec)
    .regression(inputs, answers, loss=rd.MSE)
    .alter(
        rd.Cross.graph(0.5, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1),
    )
    .limit(rd.Limit.generations(GENERATIONS))
)


def render(graph, size: int) -> Image.Image:
    outputs = graph.eval(coords(size))
    pixels = (np.clip(np.array(outputs, dtype=np.float32), 0.0, 1.0) * 255).astype(
        np.uint8
    )
    return Image.fromarray(pixels.reshape(size, size, 3), mode="RGB")


result = engine.run(log=True)
print(result)

final = render(result.value(), RENDER_SIZE)
final.save(OUT / "final.png")
print(f"Saved {OUT / 'final.png'}")
