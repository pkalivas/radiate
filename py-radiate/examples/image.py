import math

import numpy as np
import radiate as rd

from PIL import Image, ImageDraw, ImageChops, ImageStat


NUM_POLYGONS = 175
POLYGON_SIZE = 5
SEED = 18
STEPS = 20

rd.random.seed(SEED)


# Convenience to draw this polygon onto a PIL RGBA canvas (with alpha)
def draw(alleles: np.ndarray, canvas: Image.Image) -> None:
    r = alleles[0]
    g = alleles[1]
    b = alleles[2]
    a = alleles[3]

    pts = []
    for i in range(4, len(alleles), 2):
        x = alleles[i]
        y = alleles[i + 1]
        pts.append((x, y))

    if len(pts) < 4:
        return

    # Dedup consecutive identical points
    dedup = []
    for p in pts:
        if not dedup or dedup[-1] != p:
            dedup.append(p)
    pts = dedup

    w, h = canvas.size
    xy = [(int(x * (w - 1)), int(y * (h - 1))) for (x, y) in pts]

    # Convert normalized color to 0..255
    rgba = (
        int(r * 255.0 + 0.5),
        int(g * 255.0 + 0.5),
        int(b * 255.0 + 0.5),
        int(a * 255.0 + 0.5),
    )

    # Draw onto a temporary layer and alpha-composite
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer, "RGBA")
    draw.polygon(xy, fill=rgba)
    canvas.alpha_composite(layer)


def render_chromosome(width: int, height: int, genes: np.ndarray) -> Image.Image:
    canvas = Image.new("RGBA", (width, height), (255, 255, 255, 255))
    for g in genes:
        draw(g, canvas)
    return canvas


def rmse_rgb(canvas: Image.Image, target_rgb_img: Image.Image) -> float:
    """
    Compute RMSE between a rendered PIL image (canvas) and a precomputed target RGB Image.
    Uses Pillow's C-optimized ImageChops/ImageStat for speed. Alpha is ignored.
    """
    c_rgb = canvas.convert("RGB")
    if c_rgb.size != target_rgb_img.size:
        raise ValueError(
            f"Image size mismatch: canvas {c_rgb.size} vs target {target_rgb_img.size}"
        )

    diff = ImageChops.difference(c_rgb, target_rgb_img)
    stat = ImageStat.Stat(diff)
    per_channel_rms = stat.rms  # length 3 for RGB
    mse_overall = sum((v * v) for v in per_channel_rms) / 3.0
    return float(math.sqrt(mse_overall))


def run_image_evo(
    target_path: str, width: int | None = None, height: int | None = None
):
    # Load target
    target_img = Image.open(target_path).convert("RGBA")
    if width and height:
        target_img = target_img.resize((width, height), Image.LANCZOS)
    W, H = target_img.size
    target_rgb = target_img.convert("RGB")

    codec = rd.FloatCodec.matrix(
        shape=(NUM_POLYGONS, 4 + POLYGON_SIZE * 2),
        init_range=(0.0, 1.0),
        bounds=(0.0, 1.0),
        use_numpy=True,
    )

    def fitness(genes: np.ndarray) -> float:
        canvas = render_chromosome(W, H, genes)
        return rmse_rgb(canvas, target_rgb)

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness,
        objective="min",
        survivor_selector=rd.RouletteSelector(),
        offspring_selector=rd.TournamentSelector(3),
        executor=rd.Executor.WorkerPool(),
        alters=[
            rd.MeanCrossover(0.3),
            rd.JitterMutator(rate=0.01, magnitude=0.15),
            rd.UniformCrossover(0.4),
        ],
    )

    result = engine.run(rd.GenerationsLimit(STEPS), log=True)
    print(result)

    best = render_chromosome(W, H, result.value())
    best.save("py-radiate/examples/data/monalisa_output.png")


if __name__ == "__main__":
    import os
    import sys

    print("gil enabled:", sys._is_gil_enabled())
    print("python version:", os.sys.version)
    run_image_evo("py-radiate/examples/data/monalisa.png")
