
# Checkpointing

Radiate provides built-in support for checkpointing, allowing you to save the state of your genetic algorithm at regular intervals. This is particularly useful for long-running experiments, as it enables you to resume from the last checkpoint in case of interruptions. 

=== ":fontawesome-brands-python: Python"

    In python, checkpoints will be stored as pickle files (`.pkl`) in the specified directory. Each checkpoint file will be named `chckpnt_{generation}.pkl`, where `{generation}` is the generation number at which the checkpoint was taken. So below, we specify a directory to store the checkpoints in, and we specify that we want to checkpoint every 10 generations. The engine will then save the state of the engine to a checkpoint file every 10 generations in the `checks` directory.

    ```python
    --8<-- "python/misc/checkpoint_showcase.py:checkpoint"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/misc/checkpoint.rs:checkpoint"
    ```