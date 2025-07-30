#!/usr/bin/env python3
"""
"""

import radiate as rd
import numpy as np
import matplotlib.pyplot as plt  # type: ignore
import colorsys
from typing import List
import random

rd.random.set_seed(42)
np.random.seed(42)
random.seed(42)


class FractalRenderer:
    """Renders fractals from mathematical expressions."""

    def __init__(self, width: int = 800, height: int = 600, max_iter: int = 50):
        self.width = width
        self.height = height
        self.max_iter = max_iter
        self.center_x = 0.0
        self.center_y = 0.0
        self.zoom = 4.0

    def set_viewport(self, center_x: float, center_y: float, zoom: float):
        """Set the viewport for rendering."""
        self.center_x = center_x
        self.center_y = center_y
        self.zoom = zoom

    def pixel_to_complex(self, x: int, y: int) -> complex:
        """Convert pixel coordinates to complex plane coordinates."""
        real = (x - self.width / 2) / (self.width / 2) * self.zoom + self.center_x
        imag = (y - self.height / 2) / (self.height / 2) * self.zoom + self.center_y
        return complex(real, imag)

    def render_mandelbrot(self, tree) -> np.ndarray:
        """Render Mandelbrot-like fractal using evolved mathematical expression."""
        image = np.zeros((self.height, self.width, 3), dtype=np.uint8)

        for y in range(self.height):
            for x in range(self.width):
                c = self.pixel_to_complex(x, y)
                z = complex(0, 0)

                for iteration in range(self.max_iter):
                    inputs = [z.real, z.imag, c.real, c.imag, iteration / self.max_iter]
                    result = tree.eval([inputs])[0][0]
                    if isinstance(result, (list, np.ndarray)):
                        result = result[0] if len(result) > 0 else 0.0
                    z = complex(result, result)

                    if abs(z) > 2.0:
                        break

                if iteration < self.max_iter:
                    color_inputs = [
                        z.real,
                        z.imag,
                        c.real,
                        c.imag,
                        iteration / self.max_iter,
                    ]

                    color_value = tree.eval([color_inputs])[0][0]
                    if isinstance(color_value, (list, np.ndarray)):
                        color_value = (
                            color_value[0] if len(color_value) > 0 else 0.0
                        )

                    color_value = float(color_value)

                    hue = (color_value % 1.0) * 360
                    saturation = min(1.0, max(0.0, (color_value * 2) % 1.0))
                    value = min(1.0, max(0.0, 1.0 - (iteration / self.max_iter)))

                    rgb = colorsys.hsv_to_rgb(hue / 360, saturation, value)
                    image[y, x] = [int(c * 255) for c in rgb]
                else:
                    image[y, x] = [0, 0, 0]

        return image


class FractalArtEvolver:
    """Main class for evolving fractal art."""

    def __init__(self):
        # Create TreeCodec with mathematical operations
        self.codec = rd.TreeCodec(
            shape=(5, 1),  # 5 inputs, 1 output
            min_depth=3,
            max_size=30,
            vertex=[
                rd.Op.add(),
                rd.Op.sub(),
                rd.Op.mul(),
                rd.Op.div(),  # Basic arithmetic
                rd.Op.sin(),
                rd.Op.cos(),
                rd.Op.tan(),  # Trigonometric functions
                rd.Op.exp(),
                rd.Op.log(),
                rd.Op.pow(),  # Exponential/logarithmic
                rd.Op.abs(),
                rd.Op.sqrt(),  # Absolute value and square root
                rd.Op.sigmoid(),
                rd.Op.tanh(),
                rd.Op.relu(),  # Activation functions
                rd.Op.max(),
                rd.Op.min(),  # Comparison functions
                rd.Op.ceil(),
                rd.Op.floor(),  # Rounding functions
                rd.Op.mish(),
                rd.Op.swish(),  # Advanced activation functions
            ],
            leaf=[
                rd.Op.var(0),  # z_real
                rd.Op.var(1),  # z_imag
                rd.Op.var(2),  # c_real
                rd.Op.var(3),  # c_imag
                rd.Op.var(4),  # iteration
            ],
            root=rd.Op.linear(),
        )

        self.renderer = FractalRenderer(width=100, height=75, max_iter=100)

    def calculate_symmetry(self, image: np.ndarray) -> float:
        """Calculate symmetry score of the fractal."""
        height, width = image.shape[:2]

        # Horizontal symmetry
        h_symmetry = 0.0
        for y in range(height // 2):
            for x in range(width):
                diff = np.linalg.norm(image[y, x] - image[height - 1 - y, x])
                h_symmetry += 1.0 - (diff / 255.0)
        h_symmetry /= (height // 2) * width

        # Vertical symmetry
        v_symmetry = 0.0
        for y in range(height):
            for x in range(width // 2):
                diff = np.linalg.norm(image[y, x] - image[y, width - 1 - x])
                v_symmetry += 1.0 - (diff / 255.0)
        v_symmetry /= height * (width // 2)

        return (h_symmetry + v_symmetry) / 2.0

    def calculate_complexity(self, image: np.ndarray) -> float:
        """Calculate complexity score based on edge detection and color variation."""
        # Convert to grayscale for edge detection
        gray = np.mean(image, axis=2)

        # Simple edge detection using convolution-like approach
        height, width = gray.shape
        edges = np.zeros_like(gray)

        edges[:, :-1] += np.abs(gray[:, 1:] - gray[:, :-1])
        edges[:-1, :] += np.abs(gray[1:, :] - gray[:-1, :])

        edge_score = np.mean(edges)
        color_variation = np.std(image)

        complexity = (edge_score / 255.0 + color_variation / 255.0) / 2.0
        return min(1.0, complexity)

    def calculate_color_harmony(self, image: np.ndarray) -> float:
        """Calculate color harmony score."""
        # Convert to HSV for better color analysis
        hsv_image = np.zeros_like(image)
        for y in range(image.shape[0]):
            for x in range(image.shape[1]):
                rgb = image[y, x] / 255.0
                hsv_image[y, x] = colorsys.rgb_to_hsv(*rgb)

        hue_values = hsv_image[:, :, 0].flatten()
        saturation_values = hsv_image[:, :, 1].flatten()

        black_white_ratio = np.sum(saturation_values < 0.1) / len(saturation_values)

        hue_diversity = len(np.unique(hue_values)) / 360.0
        saturation_diversity = np.std(saturation_values)

        harmony = (
            hue_diversity * 0.4
            + saturation_diversity * 0.4
            + (1.0 - black_white_ratio) * 0.2
        )
        return min(1.0, harmony)

    def fitness_function(self, tree) -> float:
        """Multi-objective fitness function for fractal art."""
        image = self.renderer.render_mandelbrot(tree)

        symmetry = self.calculate_symmetry(image)
        complexity = self.calculate_complexity(image)
        color_harmony = self.calculate_color_harmony(image)

        fitness = symmetry * 0.3 + complexity * 0.4 + color_harmony * 0.3

        if complexity < 0.1 or complexity > 0.9:
            fitness *= 0.5

        if 0.3 < complexity < 0.7 and symmetry > 0.5:
            fitness *= 1.2

        return fitness

    def create_engine(self) -> rd.GeneticEngine:
        """Create genetic engine for fractal evolution."""
        return rd.GeneticEngine(
            codec=self.codec,
            fitness_func=self.fitness_function,
            offspring_selector=rd.BoltzmannSelector(4),
            executor=rd.Executor.FixedSizedWorkerPool(10),
            alters=[
                rd.TreeCrossover(0.7),
                rd.HoistMutator(0.005),
            ],
        )

    def run_evolution(self, generations: int = 50, population_size: int = 30):
        """Run the evolution process."""
        engine = self.create_engine()
        return engine.run([rd.GenerationsLimit(generations)], log=True)

    def visualize_fractal(self, tree, title: str = "Evolved Fractal"):
        """Visualize a fractal with high resolution."""
        high_res_renderer = FractalRenderer(width=800, height=600)
        image = high_res_renderer.render_mandelbrot(tree)

        plt.figure(figsize=(12, 8))
        plt.imshow(image)
        plt.title(title)
        plt.axis("off")
        plt.tight_layout()
        plt.show()

        return image

    def create_gallery(self, trees: List, titles: List[str] = None):
        """Create a gallery of fractals."""
        if titles is None:
            titles = [f"Fractal {i + 1}" for i in range(len(trees))]

        n = len(trees)
        cols = min(3, n)
        rows = (n + cols - 1) // cols

        fig, axes = plt.subplots(rows, cols, figsize=(15, 5 * rows))
        if n == 1:
            axes = [axes]
        elif rows == 1:
            axes = axes.reshape(1, -1)

        for i, (tree, title) in enumerate(zip(trees, titles)):
            row = i // cols
            col = i % cols

            image = self.renderer.render_mandelbrot(tree)
            axes[row, col].imshow(image)
            axes[row, col].set_title(title)
            axes[row, col].axis("off")

        # Hide empty subplots
        for i in range(n, rows * cols):
            row = i // cols
            col = i % cols
            axes[row, col].axis("off")

        plt.tight_layout()
        plt.show()


def main():
    """Main function to run Fractal Art evolution."""
    evolver = FractalArtEvolver()
    result = evolver.run_evolution(generations=30, population_size=20)
    best_tree = result.value()

    print(result)

    _ = evolver.visualize_fractal(best_tree, "Best Evolved Fractal")


if __name__ == "__main__":
    main()
