
# Checkpointing

Radiate provides built-in support for checkpointing, allowing you to save the state of your genetic algorithm at regular intervals. This is particularly useful for long-running experiments, as it enables you to resume from the last checkpoint in case of interruptions.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    target = "Hello, Radiate!"

    def fitness_func(x: list[str]) -> int:
        return sum(1 for i in range(len(target)) if x[i] == target[i])

    engine = rd.GeneticEngine(
        codec=rd.CharCodec.vector(len(target)),
        fitness_func=fitness_func,
        offspring_selector=rd.BoltzmannSelector(4),
    )

    result = engine.run(rd.ScoreLimit(len(target)), checkpoint=(10, "checkpoint.json"))

    # load from checkpoint from generation 10
    engine = rd.GeneticEngine(
        codec=rd.CharCodec.vector(len(target)),
        fitness_func=fitness_func,
        offspring_selector=rd.BoltzmannSelector(4),
        checkpoint_path="checkpoint_10.json",
    )

    result = engine.run(rd.ScoreLimit(len(target)))
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
            .checkpoint(10, "checkpoint.json")
            .until_score(target_len)
            .last()
            .expect("No result from engine run");

        // load from checkpoint from generation 10
        let resumed_engine = GeneticEngine::builder()
            .codec(CharCodec::vector(target.len()))
            .offspring_selector(BoltzmannSelector::new(4_f32))
            .fitness_fn(fitness_fn)
            .load_checkpoint("checkpoint_10.json")
            .build();

        let resumed_result = resumed_engine.iter()
            .until_score(target_len)
            .last()
            .expect("No result from resumed engine run");
    }
    ```