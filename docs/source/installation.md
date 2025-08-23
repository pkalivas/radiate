
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
radiate=0.0.6
```

Python's radiate package does not have any optional features - it is a single package that includes all functionality.

### Rust

```toml
[dependencies]
radiate = { version = "1.2.16", features = ["gp", "serde", "rayon"] }
```

opt-in features include:

- `gp`: Enables the [genetic programming](https://en.wikipedia.org/wiki/Genetic_programming#:~:text=In%20artificial%20intelligence%2C%20genetic%20programming,to%20the%20population%20of%20programs.) features, allowing you to work with tree and graph-based representations.
- `serde`: Enables serialization and deserialization features, allowing you to save and load the Ecosystem state to/from disk. This is useful for long-running evolutionary processes or for resuming experiments.
    * Includes support for: Ecosystem, Population, Species, Phenotype, Genotype, all Chromosomes and their associated Genes, plus `gp`'s Graph<T> and Tree<T> structures.
- `rayon`: Enables parallel processing through the [Rayon](https://docs.rs/rayon/latest/rayon/) library. Radiate can run in parallel without this feature, but this is included due to its popularity and ease of use. 
    * Note that the difference in performanec between running with radiate's internal threadpool vs. Rayon is negligible for most use cases, so you can safely run without it if you prefer.

    