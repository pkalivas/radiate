use radiate::*;

pub struct TspCodex {
    num_cities: usize,
}

impl TspCodex {
    pub fn new(num_cities: usize) -> Self {
        TspCodex { num_cities }
    }
}

impl Codex<IntChromosome<i32>, Vec<i32>> for TspCodex {
    fn encode(&self) -> Genotype<IntChromosome<i32>> {
        let mut cities: Vec<usize> = (0..self.num_cities).collect();
        random_provider::shuffle(&mut cities);

        let chromeosome = IntChromosome::from_genes(
            cities
                .iter()
                .map(|&x| {
                    IntGene::from_min_max(0, self.num_cities as i32)
                        .with_bounds(self.num_cities as i32, 0)
                        .from_allele(&(x as i32))
                })
                .collect(),
        );

        Genotype::from_chromosomes(vec![chromeosome])
    }

    fn decode(&self, genotype: &Genotype<IntChromosome<i32>>) -> Vec<i32> {
        genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .map(|gene| gene.allele)
            .collect()
    }
}

fn main() {
    let num_cities = 10;
    let distance_matrix = vec![
        vec![0.0, 2.0, 9.0, 10.0, 1.0, 3.0, 7.0, 8.0, 4.0, 6.0],
        vec![2.0, 0.0, 8.0, 9.0, 3.0, 1.0, 6.0, 7.0, 5.0, 4.0],
        vec![9.0, 8.0, 0.0, 5.0, 6.0, 4.0, 3.0, 2.0, 7.0, 1.0],
        vec![10.0, 9.0, 5.0, 0.0, 2.0, 3.0, 4.0, 1.0, 6.0, 8.0],
        vec![1.0, 3.0, 6.0, 2.0, 0.0, 5.0, 8.0, 7.0, 9.0, 4.0],
        vec![3.0, 1.0, 4.0, 3.0, 5.0, 0.0, 2.0, 6.0, 8.0, 7.0],
        vec![7.0, 6.0, 3.0, 4.0, 8.0, 2.0, 0.0, 5.0, 9.0, 1.0],
        vec![8.0, 7.0, 2.0, 1.0, 7.0, 6.0, 5.0, 0.0, 3.0, 4.0],
        vec![4.0, 5.0, 7.0, 6.0, 9.0, 8.0, 9.0, 3.0, 0.0, 2.0],
        vec![6.0, 4.0, 1.0, 8.0, 4.0, 7.0, 1.0, 4.0, 2.0, 0.0],
    ];

    let copied_distance_matrix = distance_matrix.clone();

    let codex = TspCodex::new(num_cities);
    let engine = GeneticEngine::from_codex(&codex)
        .population_size(100)
        .fitness_fn(move |genotype: Vec<i32>| {
            let distance = calculate_distance(&genotype, &distance_matrix);
            Score::from_f32(distance)
        })
        .minimizing()
        .build();

    let result = engine.run(|output| {
        println!(
            "{:?}: Distance: {:?}",
            output.index,
            output.score().as_usize()
        );
        output.score().as_usize() < 20
    });

    let best_tour = result.best;
    println!("Best tour: {:?}", best_tour);
    println!(
        "Distance: {:?}",
        calculate_distance(&best_tour, &copied_distance_matrix)
    );
}

fn calculate_distance(tour: &[i32], distance_matrix: &[Vec<f32>]) -> f32 {
    let mut total_distance = 0.0;
    for i in 0..tour.len() {
        let from = tour[i];
        let to = tour[(i + 1) % tour.len()];
        total_distance += distance_matrix[from as usize][to as usize];
    }
    total_distance
}
