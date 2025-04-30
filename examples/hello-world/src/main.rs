use radiate::*;

fn main() {
    let target = "Hello, Radiate!";
    let codex = CharCodex::vector(target.len());

    let mut engine = GeneticEngine::builder()
        .codex(codex)
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .fitness_fn(|geno: Vec<char>| {
            geno.into_iter().zip(target.chars()).fold(
                0,
                |acc, (allele, targ)| {
                    if allele == targ { acc + 1 } else { acc }
                },
            )
        })
        .build();

    let result = engine.run(|ctx| {
        let best_as_string = ctx.value().iter().collect::<String>();
        println!("[ {:?} ]: {:?}", ctx.index(), best_as_string);

        ctx.score().as_usize() == target.len()
    });

    println!("{:?}", result);
}
