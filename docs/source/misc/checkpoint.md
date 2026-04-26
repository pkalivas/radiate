
# Checkpointing

Radiate provides built-in support for checkpointing, allowing you to save the state of your genetic algorithm at regular intervals. This is particularly useful for long-running experiments, as it enables you to resume from the last checkpoint in case of interruptions. 

=== ":fontawesome-brands-python: Python"

    In python, checkpoints will be stored as pickle files (`.pkl`) in the specified directory. Each checkpoint file will be named `chckpnt_{generation}.pkl`, where `{generation}` is the generation number at which the checkpoint was taken. So below, we specify a directory to store the checkpoints in, and we specify that we want to checkpoint every 10 generations. The engine will then save the state of the engine to a checkpoint file every 10 generations in the `checks` directory.

    ```python
    import radiate as rd

    target = "Hello, Radiate!"

    def fitness_func(x: list[str]) -> int:
        return sum(1 for i in range(len(target)) if x[i] == target[i])

    engine = rd.Engine.char(len(target)).fitness(fitness_func)

    result = engine.run(rd.Limit.score(len(target)), checkpoint=(10, "checks"))

    # load from checkpoint from generation 10
    engine = (
        rd.Engine.char(len(target))
        .fitness(fitness_func)
        .load_checkpoint("checks/chckpnt_10.pkl")
    )
    
    result_from_checkpoint = engine.run(rd.Limit.score(len(target)))
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    fn main() {
        let target = "Hello, Radiate!";
        let target_len = target.len();

        fn fitness_fn(geno: Vec<char>) -> Score {
            geno.into_iter().zip(target.chars()).fold(
                0,
                |acc, (allele, targ)| {
                    if allele == targ { acc + 1 } else { acc }
                },
            ).into()
        }

        let engine = GeneticEngine::builder()
            .codec(CharCodec::vector(target.len()))
            .offspring_selector(BoltzmannSelector::new(4_f32))
            .fitness_fn(fitness_fn)
            .build();

        let result = engine.iter()
            .checkpoint(10, "checks")
            .until_score(target_len)
            .last()
            .expect("No result from engine run");

        // load from checkpoint from generation 10
        let resumed_engine = GeneticEngine::builder()
            .codec(CharCodec::vector(target.len()))
            .offspring_selector(BoltzmannSelector::new(4_f32))
            .fitness_fn(fitness_fn)
            .load_checkpoint("checks/chckpnt_10.json")
            .build();

        let resumed_result = resumed_engine.iter()
            .until_score(target_len)
            .last()
            .expect("No result from resumed engine run");
    }
    ```