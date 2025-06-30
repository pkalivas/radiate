#[cfg(test)]
mod engine_tests {
    use radiate_core::{IntCodec, problem::NoveltySearch};
    use radiate_engines::*;

    #[test]
    fn engine_can_minimize() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score_equal(0).unwrap();

        let best = result.value();
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let mut engine = GeneticEngine::builder()
            .codec(IntCodec::vector(5, 0..101))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 500);

        assert_eq!(result.value().iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];

        let mut engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(target.len(), 0..10))
            .fitness_fn(move |geno: Vec<i32>| {
                let mut score = 0;
                for i in 0..geno.len() {
                    score += (geno[i] - target[i]).abs();
                }
                score
            })
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        assert_eq!(result.value(), &vec![1, 2, 3, 4, 5]);
    }

    use radiate_core::{
        Chromosome, Codec, Gene, Genotype, Problem, Score, genome::FloatChromosome, random_provider,
    };
    use std::sync::Arc;

    /// Test problem: Evolve functions that produce diverse output patterns
    /// We want to find functions that behave differently, not necessarily optimally
    #[derive(Clone)]
    pub struct FunctionDiversityProblem {
        pub codec: FloatCodec<Vec<f32>>,
        pub test_inputs: Vec<f32>,
    }

    impl FunctionDiversityProblem {
        pub fn new(codec: FloatCodec<Vec<f32>>, test_inputs: Vec<f32>) -> Self {
            Self { codec, test_inputs }
        }
    }

    impl Problem<FloatChromosome, Vec<f32>> for FunctionDiversityProblem {
        fn encode(&self) -> Genotype<FloatChromosome> {
            self.codec.encode()
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
            self.codec.decode(genotype)
        }

        fn eval(&self, individual: &Genotype<FloatChromosome>) -> Score {
            let weights = self.decode(individual);

            // Create a simple function: f(x) = sum(weights[i] * x^i)
            let outputs: Vec<f32> = self
                .test_inputs
                .iter()
                .map(|&x| {
                    weights
                        .iter()
                        .enumerate()
                        .map(|(i, &w)| w * x.powi(i as i32))
                        .sum()
                })
                .collect();

            // Fitness: how close to a target pattern (but this is secondary to novelty)
            let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
            let fitness = 1.0
                / (1.0
                    + outputs
                        .iter()
                        .zip(target.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>());

            Score::from(fitness)
        }
    }

    unsafe impl Send for FunctionDiversityProblem {}
    unsafe impl Sync for FunctionDiversityProblem {}

    /// Distance function: Euclidean distance between output vectors
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        let sum_squared_diff: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();

        sum_squared_diff.sqrt()
    }

    /// Test novelty search
    #[test]
    fn test_novelty_search() {
        use radiate_core::*;

        random_provider::set_seed(42);

        let test_inputs = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let codec = FloatCodec::vector(5, -100.0..100.0);

        let base_problem = FunctionDiversityProblem::new(codec.clone(), test_inputs);

        // let search = NoveltySearch::new(|vals| vals.clone(), euclidean_distance, 10, 0.03);

        let base_population = (0..100)
            .map(|_| Phenotype::from((base_problem.encode(), 0)))
            .collect::<Population<FloatChromosome>>();
        let second_population = base_population
            .iter()
            .map(|ind| ind.clone())
            .collect::<Population<FloatChromosome>>();

        let regular_engine = GeneticEngine::builder()
            .problem(base_problem.clone())
            .population(base_population)
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(EliteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .minimizing()
            .fitness_fn(|geno: Vec<f32>| {
                let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
                1.0 / (1.0
                    + geno
                        .iter()
                        .zip(target.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>())
            })
            .build();

        let novelty_engine = GeneticEngine::builder()
            .codec(codec)
            .population(second_population)
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(EliteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .minimizing()
            .fitness_fn(NoveltySearch::new(
                |vals: &Vec<f32>| vals.clone(),
                euclidean_distance,
                10,
                0.03,
            ))
            .build();

        let regular_generation = regular_engine.iter().take(20).last().unwrap();
        let novelty_generation = novelty_engine.iter().take(20).last().unwrap();

        println!("{:?}", regular_generation);
        println!("{:?}", novelty_generation);

        println!(
            "Regular evolution best fitness: {:?}",
            regular_generation.score()
        );

        println!(
            "Novelty search best fitness: {:?}",
            novelty_generation.score()
        );

        let regular_diversity = calculate_diversity(regular_generation.population());
        let novelty_diversity = calculate_diversity(novelty_generation.population());

        println!("Regular evolution diversity: {}", regular_diversity);
        println!("Novelty search diversity: {}", novelty_diversity);

        assert!(novelty_diversity > regular_diversity);
    }

    fn calculate_diversity(population: &Population<FloatChromosome>) -> f32 {
        let descriptors: Vec<Vec<f32>> = population
            .iter()
            .map(|individual| {
                println!("SCORE: {:?}", individual.score());
                let genotype = individual.genotype();
                genotype
                    .iter()
                    .flat_map(|chromosome| chromosome.iter().map(|g| *g.allele()))
                    .collect()
            })
            .collect();

        if descriptors.is_empty() {
            return 0.0;
        }

        let dimension = descriptors[0].len();
        let mut total_range = 0.0;

        for d in 0..dimension {
            let values: Vec<f32> = descriptors
                .iter()
                .map(|desc| desc.get(d).unwrap_or(&0.0))
                .copied()
                .collect();

            let min_val = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            total_range += max_val - min_val;
        }

        total_range / (dimension as f32 * 200.0)
    }
}

// use std::collections::VecDeque;
// use std::sync::{Arc, RwLock};
// use rand::{Rng, thread_rng};

// // Agent's genome: a sequence of moves
// #[derive(Clone, Debug)]
// struct Agent {
//     genome: Vec<Move>,
// }

// #[derive(Clone, Copy, Debug)]
// enum Move { Up, Down, Left, Right }

// impl Agent {
//     fn random(length: usize) -> Self {
//         let mut rng = thread_rng();
//         let moves = (0..length).map(|_| {
//             match rng.gen_range(0..4) {
//                 0 => Move::Up,
//                 1 => Move::Down,
//                 2 => Move::Left,
//                 _ => Move::Right,
//             }
//         }).collect();
//         Agent { genome: moves }
//     }

//     fn position(&self, maze: &Maze) -> (usize, usize) {
//         let mut pos = maze.start;
//         for m in &self.genome {
//             pos = maze.step(pos, *m);
//         }
//         pos
//     }

//     fn descriptor(&self, maze: &Maze) -> Vec<f32> {
//         let (x, y) = self.position(maze);
//         vec![x as f32, y as f32]
//     }
// }

// struct Maze {
//     width: usize,
//     height: usize,
//     walls: Vec<(usize, usize)>,
//     start: (usize, usize),
//     goal: (usize, usize),
// }

// impl Maze {
//     fn step(&self, pos: (usize, usize), mv: Move) -> (usize, usize) {
//         let (x, y) = pos;
//         let new_pos = match mv {
//             Move::Up => (x, y.saturating_sub(1)),
//             Move::Down => (x, (y + 1).min(self.height - 1)),
//             Move::Left => (x.saturating_sub(1), y),
//             Move::Right => ((x + 1).min(self.width - 1), y),
//         };
//         if self.walls.contains(&new_pos) { pos } else { new_pos }
//     }

//     fn distance_to_goal(&self, pos: (usize, usize)) -> f32 {
//         let dx = (pos.0 as isize - self.goal.0 as isize).abs();
//         let dy = (pos.1 as isize - self.goal.1 as isize).abs();
//         (dx + dy) as f32
//     }
// }

// // Simple novelty objective
// struct Novelty {
//     archive: Arc<RwLock<VecDeque<Vec<f32>>>>,
//     k: usize,
// }

// impl Novelty {
//     fn new(k: usize) -> Self {
//         Novelty {
//             archive: Arc::new(RwLock::new(VecDeque::new())),
//             k,
//         }
//     }

//     fn eval(&self, descriptor: Vec<f32>) -> f32 {
//         let archive = self.archive.read().unwrap();
//         let mut dists = archive.iter()
//             .map(|past| euclidean(&descriptor, past))
//             .collect::<Vec<f32>>();

//         dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
//         let k = self.k.min(dists.len());
//         let novelty = if k > 0 {
//             dists.iter().take(k).sum::<f32>() / (k as f32)
//         } else {
//             1.0
//         };

//         drop(archive);
//         self.archive.write().unwrap().push_back(descriptor);
//         novelty
//     }
// }

// fn euclidean(a: &[f32], b: &[f32]) -> f32 {
//     a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt()
// }

// fn main() {
//     let maze = Maze {
//         width: 10,
//         height: 10,
//         walls: vec![(2, 1), (2, 2), (2, 3), (2, 4), (2, 5)], // Trap wall
//         start: (0, 0),
//         goal: (9, 9),
//     };

//     let novelty = Novelty::new(5);
//     let mut population: Vec<Agent> = (0..100).map(|_| Agent::random(20)).collect();

//     for generation in 0..100 {
//         population.sort_by(|a, b| {
//             let na = novelty.eval(a.descriptor(&maze));
//             let nb = novelty.eval(b.descriptor(&maze));
//             nb.partial_cmp(&na).unwrap()
//         });

//         println!("Gen {}: Best novelty = {:.3}", generation, novelty.eval(population[0].descriptor(&maze)));

//         // Simple replacement
//         let elites = &population[0..10];
//         let mut rng = thread_rng();
//         population = elites.iter().cloned().collect();
//         while population.len() < 100 {
//             let mut child = elites[rng.gen_range(0..elites.len())].clone();
//             let idx = rng.gen_range(0..child.genome.len());
//             child.genome[idx] = match rng.gen_range(0..4) {
//                 0 => Move::Up,
//                 1 => Move::Down,
//                 2 => Move::Left,
//                 _ => Move::Right,
//             };
//             population.push(child);
//         }
//     }
// }
