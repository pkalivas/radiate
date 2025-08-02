use radiate::*;

const N_QUEENS: usize = 32;

fn main() {
    random_provider::set_seed(500);

    let engine = GeneticEngine::builder()
        .codec(IntChromosome::from((N_QUEENS, 0..N_QUEENS as i8)))
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4.0))
        .crossover(MultiPointCrossover::new(0.75, 2))
        .mutator(UniformMutator::new(0.05))
        .fitness_fn(|queens: Vec<i8>| {
            let mut score = 0;

            for i in 0..N_QUEENS {
                for j in (i + 1)..N_QUEENS {
                    if queens[i] == queens[j] {
                        score += 1;
                    }
                    if (i as i8 - j as i8).abs() == (queens[i] - queens[j]).abs() {
                        score += 1;
                    }
                }
            }

            score
        })
        .build();

    let result = engine.iter().logging().until_score(0).last().unwrap();

    println!("Best Score: {:?}", result);

    println!("\nResult Queens Board ({:.3?}):", result.time());

    let board = &result.value();
    for i in 0..N_QUEENS {
        for j in 0..N_QUEENS {
            if board[j] == i as i8 {
                print!("Q ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
}
