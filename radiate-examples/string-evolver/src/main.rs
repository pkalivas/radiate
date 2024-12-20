use radiate::*;

fn main() {
    let target = "Hello, Radiate!";
    let codex = CharCodex::new(1, target.len());

    let engine = GeneticEngine::from_codex(&codex)
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .survivor_selector(TournamentSelector::new(3))
        .alter(alters![
            UniformMutator::new(0.1),
            UniformCrossover::new(0.5)
        ])
        .fitness_fn(|genotype: Vec<Vec<char>>| {
            Score::from_usize(genotype.into_iter().flatten().zip(target.chars()).fold(
                0,
                |acc, (geno, targ)| {
                    if geno == targ {
                        acc + 1
                    } else {
                        acc
                    }
                },
            ))
        })
        .build();

    let result = engine.run(|output| {
        let best_as_string = output.best[0].iter().collect::<String>();
        println!("[ {:?} ]: {:?}", output.index, best_as_string);

        output.score().as_usize() == target.len()
    });

    println!("{:?}", result);
}
