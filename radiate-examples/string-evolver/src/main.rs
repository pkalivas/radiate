use radiate::*;

fn main() {
    let target = "Chicago, IL is the best city in the world!";
    let codex = CharCodex::new(1, target.len());

    let engine =
        GeneticEngine::from_codex(&codex)
            .offspring_selector(RankSelector::new())
            .survivor_selector(TournamentSelector::new(3))
            .alterer(vec![Alterer::Mutator(0.01), Alterer::UniformCrossover(0.5)])
            .fitness_fn(|genotype: String| {
                Score::from_usize(genotype.chars().zip(target.chars()).fold(
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
        println!("[ {:?} ]: {:?}", output.index, output.best);

        output.score().as_usize() == target.len()
    });

    println!("{:?}", result);
}
