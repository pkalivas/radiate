use radiate::*;

fn main() {
    let target = "Hello, Radiate!";
    let codex = CharCodex::new(1, target.len());

    let engine = GeneticEngine::from_codex(&codex)
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .fitness_fn(|geno: Vec<Vec<char>>| {
            geno.into_iter()
                .flatten()
                .zip(target.chars())
                .fold(
                    0,
                    |acc, (geno, targ)| {
                        if geno == targ {
                            acc + 1
                        } else {
                            acc
                        }
                    },
                )
        })
        .build();

    let result = engine.run(|ctx| {
        let best_as_string = ctx.best.iter().flatten().collect::<String>();
        println!("[ {:?} ]: {:?}", ctx.index, best_as_string);

        ctx.score().as_usize() == target.len()
    });

    println!("{:?}", result);
}
