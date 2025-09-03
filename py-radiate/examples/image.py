# image_evo_anygene.py
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
    dedup= []
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
        objectives="min",
        survivor_selector=rd.RouletteSelector(),
        offspring_selector=rd.TournamentSelector(3),
        alters=[
            rd.MeanCrossover(0.3),
            rd.JitterMutator(rate=0.01, magnitude=0.15),
            rd.UniformCrossover(0.4),
        ],
    )

    result = engine.run(rd.GenerationsLimit(STEPS), log=True)
    print(result)

    best = render_chromosome(W, H, result.value())
    best.save("examples/data/monalisa_output.png")


if __name__ == "__main__":
    run_image_evo("examples/data/monalisa.png")


# # -------------------------
# # AnyGene: PolygonGene
# # -------------------------
# class PolygonGene(rd.AnyGene):
#     """
#     AnyGene-backed polygon:
#       - color in normalized floats [0,1]
#       - 'points': list[(x,y)] normalized to [0,1]
#     """

#     def __init__(self, n: int, width: int, height: int):
#         self.n = n
#         self.width = width
#         self.height = height

#         # RGBA in [0,1], with alpha biased toward transparency
#         self.r = rd.random.float()
#         self.g = rd.random.float()
#         self.b = rd.random.float()
#         a_base = rd.random.float() * rd.random.float()
#         self.a = max(0.2, a_base)

#         # Correlated random walk for points, normalized [0,1]
#         px = rd.random.float()
#         py = rd.random.float()
#         pts = []
#         for _ in range(n):
#             px += rd.random.float() - 0.5
#             py += rd.random.float() - 0.5
#             px = min(1.0, max(0.0, px))
#             py = min(1.0, max(0.0, py))
#             pts.append((px, py))
#         self.points = pts

#     def __factory__(self):
#         return PolygonGene(self.n, self.width, self.height)

#     def __repr__(self) -> str:
#         return f"PolygonGene(n={self.n}, width={self.width}, height={self.height}, points={self.points})"

#     # Convenience to draw this polygon onto a PIL RGBA canvas (with alpha)
#     def draw(self, canvas: Image.Image) -> None:
#         if len(self.points) < 3:
#             return

#         # Dedup consecutive identical points
#         pts = []
#         for p in self.points:
#             if not pts or pts[-1] != p:
#                 pts.append(p)
#         if len(pts) < 3:
#             return
#         if pts[0] == pts[-1]:
#             pts = pts[:-1]
#         if len(pts) < 3:
#             return

#         w, h = canvas.size
#         xy = [(int(x * (w - 1)), int(y * (h - 1))) for (x, y) in pts]

#         # Convert normalized color to 0..255
#         rgba = (
#             int(self.r * 255.0 + 0.5),
#             int(self.g * 255.0 + 0.5),
#             int(self.b * 255.0 + 0.5),
#             int(self.a * 255.0 + 0.5),
#         )

#         # Draw onto a temporary layer and alpha-composite
#         layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
#         draw = ImageDraw.Draw(layer, "RGBA")
#         draw.polygon(xy, fill=rgba)
#         canvas.alpha_composite(layer)


# # -------------------------
# # Chromosome renderer
# # -------------------------
# def render_chromosome(width: int, height: int, genes: list[PolygonGene]) -> Image.Image:
#     canvas = Image.new("RGBA", (width, height), (255, 255, 255, 255))
#     for g in genes:
#         g.draw(canvas)
#     return canvas


# # -------------------------
# # Fitness (mirror Rust eval)
# # -------------------------
# def rmse_rgb(canvas: Image.Image, target: Image.Image) -> float:
#     # Both RGBA. Compare only RGB (ignore alpha)
#     c = canvas.convert("RGBA")
#     t = target.convert("RGBA")
#     cw, ch = c.size
#     tw, th = t.size
#     assert (cw, ch) == (tw, th)

#     c_px = c.tobytes()
#     t_px = t.tobytes()
#     # RGBA stride = 4
#     total = 0.0
#     count = cw * ch * 3
#     for i in range(0, len(c_px), 4):
#         dr = t_px[i + 0] - c_px[i + 0]
#         dg = t_px[i + 1] - c_px[i + 1]
#         db = t_px[i + 2] - c_px[i + 2]
#         total += float(dr * dr + dg * dg + db * db)
#     mse = total / float(count)
#     return math.sqrt(mse)


# # -------------------------
# # Mutator (mirror ImageMutator)
# # -------------------------
# class ImageMutator(rd.Mutator):
#     """
#     Mutates each float in the polygon with probability 'rate'
#     by adding uniform(-magnitude, +magnitude) then clamping to [0,1].
#     """

#     def __init__(self, rate: float, magnitude: float):
#         super().__init__(rate)
#         self.rate = rate
#         self.magnitude = magnitude

#     def mutate(self, candidate: rd.Chromosome) -> rd.Chromosome:
#         def jitter(g: dict) -> dict:
#             if rd.random.float() > self.rate:
#                 return None

#             colors = ["r", "g", "b", "a"]

#             for color in colors:
#                 if rd.random.float() < self.rate:
#                     g[color] = min(
#                         1.0,
#                         max(
#                             0.0,
#                             g[color] + (rd.random.float() * 2 - 1) * self.magnitude,
#                         ),
#                     )
#             # points
#             pts = []
#             for x, y in g["points"]:
#                 dx = (
#                     (rd.random.float() * 2 - 1) * self.magnitude
#                     if rd.random.float() < self.rate
#                     else 0
#                 )
#                 dy = (
#                     (rd.random.float() * 2 - 1) * self.magnitude
#                     if rd.random.float() < self.rate
#                     else 0
#                 )
#                 pts.append(
#                     (
#                         min(1.0, max(0.0, x + dx)),
#                         min(1.0, max(0.0, y + dy)),
#                     )
#                 )
#             return {**g, "points": pts}

#         candidate.apply(jitter)

#         return candidate


# # -------------------------
# # Custom PolygonMeanCrossover
# # -------------------------
# class PolygonMeanCrossover(rd.Crossover):
#     def __init__(self, rate: float):
#         super().__init__(rate)

#     def crossover(
#         self, left: rd.Chromosome, right: rd.Chromosome
#     ) -> tuple[rd.Chromosome, rd.Chromosome]:
#         """
#         Averages color and point coordinates while preserving the original AnyGene dict schema.
#         Writes a *biased* mean into each side to keep some diversity:
#           - left := mean(a,b)
#           - right := mean(b,a)   (same mean here, but code allows asymmetric blending if extended)
#         """

#         def clamp01(x: float) -> float:
#             if x < 0.0:
#                 return 0.0
#             if x > 1.0:
#                 return 1.0
#             return x

#         def mean_dict(pa: dict, pb: dict) -> dict:
#             # Average colors
#             r = clamp01(0.5 * (pa["r"] + pb["r"]))
#             g = clamp01(0.5 * (pa["g"] + pb["g"]))
#             b = clamp01(0.5 * (pa["b"] + pb["b"]))
#             a = clamp01(0.5 * (pa["a"] + pb["a"]))

#             # Average points (up to min length), then keep the tail from pa to preserve size
#             pts_a = pa["points"]
#             pts_b = pb["points"]
#             m = min(len(pts_a), len(pts_b))
#             pts = []
#             for j in range(m):
#                 ax, ay = pts_a[j]
#                 bx, by = pts_b[j]
#                 pts.append((clamp01(0.5 * (ax + bx)), clamp01(0.5 * (ay + by))))
#             if len(pts_a) > m:
#                 # keep remaining points from pa (already in [0,1])
#                 pts.extend(pts_a[m:])

#             # IMPORTANT: preserve all existing keys (incl. '__class__', 'n', 'width', 'height', etc.)
#             # and only overwrite the averaged ones.
#             return {**pa, "r": r, "g": g, "b": b, "a": a, "points": pts}

#         n = min(len(left), len(right))
#         for i in range(n):
#             if rd.random.float() < self.rate:
#                 # Snapshot parent alleles BEFORE applying, to avoid self-referential updates
#                 a = dict(left[i].allele())
#                 b = dict(right[i].allele())

#                 left.view(i).apply(lambda _g, pa=a, pb=b: mean_dict(pa, pb))
#                 # right[i].apply(lambda _g, pa=b, pb=a: mean_dict(pa, pb))
#         return left, right


# # -------------------------
# # Engine wiring
# # -------------------------
# def run_image_evo(
#     target_path: str, width: int | None = None, height: int | None = None
# ):
#     # Load target
#     target_img = Image.open(target_path).convert("RGBA")
#     if width and height:
#         target_img = target_img.resize((width, height), Image.LANCZOS)
#     W, H = target_img.size

#     # Gene factory (used by AnyCodec to build initial chromosome and by Rust for fresh instances)
#     def make_gene() -> PolygonGene:
#         return PolygonGene(POLYGON_SIZE, W, H)

#     codec = rd.AnyCodec(NUM_POLYGONS, make_gene)

#     # Fitness: decode -> render -> RMSE to target (minimize)
#     def fitness(genes: list[PolygonGene]) -> float:
#         canvas = render_chromosome(W, H, genes)
#         return rmse_rgb(canvas, target_img)

#     engine = rd.GeneticEngine(
#         codec=codec,
#         fitness_func=fitness,
#         objectives="min",
#         survivor_selector=rd.RouletteSelector(),
#         offspring_selector=rd.TournamentSelector(3),
#         alters=[
#             PolygonMeanCrossover(0.3),
#             ImageMutator(rate=0.08, magnitude=0.15),
#             rd.UniformCrossover(0.4),
#         ],
#     )

#     result = engine.run(rd.GenerationsLimit(STEPS), log=True)
#     print(result)

#     best = render_chromosome(W, H, result.value())
#     best.save("examples/data/monalisa_output.png")
#     print("Saved output.png")


# if __name__ == "__main__":
#     run_image_evo("examples/data/monalisa.png")
