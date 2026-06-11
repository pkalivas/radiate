#!/usr/bin/env python3
"""
Radiate example: Evolisa (a.k.a. "Mona Lisa with polygons")

Evolve a set of semi-transparent triangles to approximate a target image.
Each triangle is one chromosome of 10 genes -- 3 vertices (x, y) plus RGBA --
all normalized to [0, 1]. Fitness is the mean-squared pixel error between the
rendered candidate and the (downscaled) target; we minimize it.

Requires: pillow, numpy  (`uv pip install pillow numpy`)
"""

import sys
from pathlib import Path

import numpy as np  # type: ignore
from PIL import Image, ImageDraw  # type: ignore

import radiate as rd

# --- config -----------------------------------------------------------------
POLYGONS = 175  # number of polygons (Rust: NUM_GENES)
VERTS = 5  # vertices per polygon (Rust: POLYGON_SIZE)
GENES_PER = 4 + 2 * VERTS  # [r, g, b, a, x0, y0, ... ] = 14 floats
RENDER_MAX = 128  # longest side used for fitness (small = fast)
SAVE_EVERY = 25  # snapshot cadence (generations)
GENERATIONS = 1000  # Rust: .iter().take(1000)

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
    """genes: (POLYGONS, GENES_PER) in [0, 1] -> composited RGB image.

    Layout per polygon mirrors the Rust example: [r, g, b, a, x0, y0, ...].
    White background + per-polygon alpha compositing, same as `Polygon::draw`.
    """
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


def fit(chromosomes: list[np.ndarray]) -> float:
    genes = np.asarray(chromosomes, dtype=np.float32)  # (POLYGONS, GENES_PER)
    candidate = np.asarray(render(genes, W, H), dtype=np.float32)
    return float(np.sqrt(np.mean((candidate - target_arr) ** 2)))  # RMS, like Rust


# --- engine -----------------------------------------------------------------
class ImageWriter(rd.EventHandler):
    def __init__(self, save_every: int, out_dir: Path):
        super().__init__(rd.EventType.EPOCH_COMPLETE)
        self.save_every = save_every
        self.out_dir = out_dir

    def on_event(self, event: rd.EngineEvent):
        if event.index() % self.save_every == 0:
            print(event)
            best = np.asarray(event.value(), dtype=np.float32)
            frame = render(best, W, H)
            frame.save(self.out_dir / f"gen_{event.index():05d}.png")
            print(f"gen {event.index():5d}  rms={event.score()}")


engine = (
    rd.Engine.float(
        shape=[GENES_PER] * POLYGONS,  # one chromosome per polygon
        init_range=(0.0, 1.0),
        bounds=(0.0, 1.0),  # coords + colors stay normalized
        use_numpy=True,
        dtype=rd.Float32,
    )
    .fitness(fit)
    .minimizing()
    .select(survivor=rd.Select.roulette(), offspring=rd.Select.tournament(3))
    .alters(
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


# --- drive + capture frames -------------------------------------------------
# frames: list[Image.Image] = []
# for epoch in engine:
#     if epoch.index() % SAVE_EVERY == 0:
#         best = np.asarray(epoch.value(), dtype=np.float32)
#         frame = render(best, W, H)
#         frame.save(OUT / f"gen_{epoch.index():05d}.png")
#         frames.append(frame.copy())
#         print(f"gen {epoch.index():5d}  rms={epoch.score()}")

# if frames:
#     frames[0].save(
#         OUT / "evolisa.gif",
#         save_all=True,
#         append_images=frames[1:],
#         duration=80,
#         loop=0,
#     )
#     print(f"\nWrote {len(frames)} frames + {OUT / 'evolisa.gif'}")
