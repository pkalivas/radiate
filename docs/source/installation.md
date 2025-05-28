
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
radiate=0.0.2
```

!!! warning ":construction: Under Construction :construction:"

    The features for Python are still in development and will be available in future releases.


### Rust

```toml
[dependencies]
radiate = { version = "1.2.12", features = ["gp"] }
```

opt-in features include:

- `gp`: Enables the [genetic programming](https://en.wikipedia.org/wiki/Genetic_programming#:~:text=In%20artificial%20intelligence%2C%20genetic%20programming,to%20the%20population%20of%20programs.) features, allowing you to work with tree and graph-based representations.
    