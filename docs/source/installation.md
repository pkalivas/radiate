
## Installing

Installing Radiate is straightforward. Below are the instructions for each language - use whichever applicable package manager you prefer.

=== ":fontawesome-brands-python: Python"

    ```bash
    pip install radiate
    ```

=== ":fontawesome-brands-rust: Rust"
    ```shell
    cargo add radiate -F gp

    # Or Cargo.toml
    [dependencies]
    radiate = { version = "x", features = ["gp", ...] }
    ```

## Importing

To use Radiate, simply import it in your project as such:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;
    ```

## Feature Flags

By installing the above, you will get the core library. However, Radiate has a few optional features that you can enable to extend its functionality.

### Python

```text
# requirements.txt
radiate=0.0.14
```

Python's radiate package includes all rust features by default, so you get all of rust's features (gp, serde, rayon, ui) without needing to do anything extra. That being said, radiate's python bindings do include their own set of features which futher integrate it into the python ecosystem at large. All of the following have internal checks within radiate and some specialized classes to work directly with these features. Radiate's python bindings include the following features:

- `polars`: Enables integration with the [Polars](https://www.pola.rs/) DataFrame library, allowing you to easily convert and manipulate data collected from the engine in a tabular format. This is especially useful for analyzing and visualizing metrics collected during evolution.
- `pandas`: Enables integration with the [Pandas](https://pandas.pydata.org/) DataFrame library, providing similar functionality to the `polars` feature but with support for Pandas' extensive data manipulation capabilities. This allows for seamless analysis and visualization of evolutionary metrics using the popular Pandas library.
- `torch`: Enables integration with the [PyTorch](https://pytorch.org/) library, allowing you to easily convert input data and genome evalutation to/from PyTorch tensors.
- `numpy`: Enables integration with the [NumPy](https://numpy.org/) library, allowing you to easily convert genoms, fitness evaluations, and collected metrics to/from NumPy arrays for efficient numerical computations and analysis.
- `matplotlib`: Enables integration with the [Matplotlib](https://matplotlib.org/) library, allowing you to easily visualize metrics collected from the engine using Matplotlib's powerful plotting capabilities. 
- `all`: Enables all of the above features.

```bash
uv add "radiate[all]" # to enable all features
```

### Rust

```toml
[dependencies]
# Include the radiate crate with all optional features enabled.
radiate = { version = "1.3.0", features = ["gp", "serde", "rayon", "ui"] }
```

opt-in features include:

- `gp`: Enables the [genetic programming](https://en.wikipedia.org/wiki/Genetic_programming#:~:text=In%20artificial%20intelligence%2C%20genetic%20programming,to%20the%20population%20of%20programs.) features, allowing you to work with tree and graph-based representations.
- `serde`: Enables serialization and deserialization features, allowing you to save and load the Ecosystem state to/from disk. This is useful for long-running evolutionary processes or for resuming experiments.
    * Includes support for: Ecosystem, Population, Species, Phenotype, Genotype, all Chromosomes and their associated Genes, plus `gp`'s Graph<T> and Tree<T> structures.
- `rayon`: Enables parallel processing through the [Rayon](https://docs.rs/rayon/latest/rayon/) library. Radiate can run in parallel without this feature, but this is included due to its popularity and ease of use. 
    * Note that the difference in performance between running with radiate's internal threadpool vs. Rayon is negligible for most use cases, so you can safely run without it if you prefer.
- `ui`: This feature enables a simple terminal command-line user interface (TUI) for monitoring and controlling (pause/resume/step) evolutionary runs. It provides real-time feedback on the progress of the evolution, including a plethora of statistics and visualizations. Big thanks to [ratatui](https://ratatui.rs). 