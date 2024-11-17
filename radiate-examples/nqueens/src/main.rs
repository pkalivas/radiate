use radiate::*;

const N_QUEENS: usize = 8;

fn main() {
    let codex = IntCodex::<i8>::new(1, N_QUEENS, 0, N_QUEENS as i8);

    let engine = GeneticEngine::from_codex(&codex)
        .minimizing()
        .num_threads(10)
        .offspring_selector(RankSelector::new())
        .alterer(vec![
            Alterer::SinglePointCrossover(0.5),
            Alterer::Mutator(0.01),
        ])
        .fitness_fn(|genotype: Vec<Vec<i8>>| {
            let queens = &genotype[0];
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

            Score::from_usize(score)
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_usize());

        output.score().as_usize() == 0
    });

    println!("\nResult Queens Board ({:.3?}):", result.timer.elapsed());

    let board = &result.best[0];
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