use radiate::prelude::*;

// --8<-- [start:checkpoint]
const TARGET: &str = "Hello, Radiate!";

fn main() {
    let target_len = TARGET.len();

    fn fitness_fn(geno: Vec<char>) -> Score {
        geno.into_iter()
            .zip(TARGET.chars())
            .fold(
                0,
                |acc, (allele, targ)| if allele == targ { acc + 1 } else { acc },
            )
            .into()
    }

    let engine = GeneticEngine::builder()
        .codec(CharCodec::vector(TARGET.len()))
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .fitness_fn(fitness_fn)
        .build();

    let result = engine
        .iter()
        .checkpoint(10, "checks")
        .until_score(target_len)
        .last()
        .expect("No result from engine run");

    // load from checkpoint from generation 10
    let resumed_engine = GeneticEngine::builder()
        .codec(CharCodec::vector(TARGET.len()))
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .fitness_fn(fitness_fn)
        .load_checkpoint("checks/chckpnt_10.json", JsonReader)
        .build();

    let resumed_result = resumed_engine
        .iter()
        .until_score(target_len)
        .last()
        .expect("No result from resumed engine run");
}
// --8<-- [end:checkpoint]
