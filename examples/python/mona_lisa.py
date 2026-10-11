#!/usr/bin/env python3
"""
Radiate example: Evolisa (a.k.a. "Mona Lisa with polygons")

Evolve a set of semi-transparent polygons to approximate a target image.
Each polygon is one chromosome of 4 + 2 * VERTS genes -- VERTS vertices (x, y) plus RGBA --
all normalized to [0, 1]. Fitness is the mean-squared pixel error between the
rendered candidate and the (downscaled) target; we minimize it.

Requires: pillow, numpy  (`uv pip install pillow numpy`)

This is _almost_ a 1:1 translation of the Rust example, but with a few differences:
- The Rust example uses a custom renderer, while this one uses Pillow.
- The Rust example uses a custom fitness function, while this one uses NumPy.

The Rust example is going to be faster. Its rust, it can run natively in parallel, we
don't have to cross the Rust-Python bridge, and we don't have to convert between Pillow/Numpy/Rust image formats.
BUT, this Python example is easier to read and modify, and it can run on any platform that supports Python. Other than
that, the two examples will produce almost the exact same results - which is pretty cool.
"""

import sys
from pathlib import Path

import numpy as np  # type: ignore
import radiate as rd
from PIL import Image, ImageDraw  # type: ignore

# --- config -----------------------------------------------------------------
POLYGONS = 175  # number of polygons (Rust: NUM_GENES)
VERTS = 5  # vertices per polygon (Rust: POLYGON_SIZE)
GENES_PER = 4 + 2 * VERTS  # [r, g, b, a, x0, y0, ... ]
RENDER_MAX = 128  # longest side used for fitness (small = fast)
SAVE_EVERY = 100  # snapshot cadence (generations) for the ImageWriter event handler
GENERATIONS = 1000

ROOT = Path(__file__).parent.parent
OUT = ROOT / "data" / "results" / "mona_lisa"
OUT.mkdir(parents=True, exist_ok=True)

rd.random.seed(50)

# --- target -----------------------------------------------------------------
target_path = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "data" / "monalisa.png"
if not target_path.exists():
    sys.exit(
        f"Target image not found: {target_path}\nPass one: python {Path(__file__).name} target.png"
    )

target_img = Image.open(target_path).convert("RGB")
scale = RENDER_MAX / max(target_img.size)
W, H = (
    max(1, round(target_img.width * scale)),
    max(1, round(target_img.height * scale)),
)
target_img = target_img.resize((W, H))
target_arr = np.asarray(target_img, dtype=np.float32)  # (H, W, 3)


# --- render + fitness -------------------------------------------------------
def render(genes: np.ndarray, w: int, h: int) -> Image.Image:
    """genes: one individual's POLYGONS * GENES_PER values in [0, 1] -> composited RGB image.

    Accepts the flat (POLYGONS * GENES_PER,) decode or (POLYGONS, GENES_PER).
    Layout per polygon mirrors the Rust example: [r, g, b, a, x0, y0, ...].
    White background + per-polygon alpha compositing.
    """
    genes = np.asarray(genes, dtype=np.float32).reshape(POLYGONS, GENES_PER)
    canvas = Image.new("RGBA", (w, h), (255, 255, 255, 255))
    for poly in genes:
        color = (
            int(poly[0] * 255),
            int(poly[1] * 255),
            int(poly[2] * 255),
            int(poly[3] * 255),
        )
        pts = [(poly[4 + 2 * k] * w, poly[5 + 2 * k] * h) for k in range(VERTS)]
        layer = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        ImageDraw.Draw(layer).polygon(pts, fill=color)
        canvas = Image.alpha_composite(canvas, layer)
    return canvas.convert("RGB")


@rd.fitness(batch=True)
def fit(population: np.ndarray) -> np.ndarray:
    """population: (N, POLYGONS * GENES_PER) -- every individual awaiting evaluation, one row each.

    Rendering is still one Pillow call per individual, but the batch crosses the
    Rust-Python bridge once and the RMS error is computed for all N in a single
    vectorized pass. Returns one score per individual, shape (N,), in input order.
    """
    candidates = np.stack(
        [np.asarray(render(member, W, H), dtype=np.float32) for member in population]
    )  # (N, H, W, 3)
    return np.sqrt(np.mean((candidates - target_arr) ** 2, axis=(1, 2, 3)))


class ImageWriter(rd.EventHandler):
    def __init__(self, save_every: int, out_dir: Path):
        super().__init__(rd.EventType.EPOCH_COMPLETE)
        self.save_every = save_every
        self.out_dir = out_dir

    def on_event(self, event: rd.EngineEvent):
        if event.index % self.save_every == 0:
            print(event)
            best = np.asarray(event.value(), dtype=np.float32)
            frame = render(best, W, H)
            frame.save(self.out_dir / f"gen_{event.index:05d}.png")
            print(f"gen {event.index:5d}  rms={event.score()}")


engine = (
    rd.Engine.float(
        shape=sum(
            [GENES_PER] * POLYGONS
        ),  # one flat chromosome, POLYGONS * GENES_PER genes
        init_range=(0.0, 1.0),
        bounds=(0.0, 1.0),  # coords + colors stay normalized
        use_numpy=True,
        dtype=rd.Float32,
    )
    .fitness(fit)
    .minimizing()
    .subscribe(ImageWriter(SAVE_EVERY, OUT))
    .select(rd.Select.tournament(3), rd.Select.roulette())
    .alter(
        rd.Cross.mean(0.3),
        rd.Mutate.jitter(0.01, 0.15),
        rd.Cross.uniform(0.4),
    )
    .limit(rd.Limit.generations(GENERATIONS))
)

if not rd._GIL_ENABLED:
    engine = engine.parallel()

result = engine.run(log=True)
value = result.value()

frame = render(np.asarray(value, dtype=np.float32), W, H)
frame.save(OUT / "final.png")
print(f"Final RMS: {result.score()}")
