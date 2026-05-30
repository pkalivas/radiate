use radiate::prelude::*;

// Stand-ins for the "plug in your own model" sections (Composite & Novelty).
// In a real program these would be your decoded domain type and its codec.
type MyDecodedModel = Vec<f32>;

#[derive(Clone)]
struct MyModel(Vec<f32>);

impl MyModel {
    fn get_behavior_vector(&self) -> Vec<f32> {
        self.0.clone()
    }
}

fn main() {
    // --8<-- [start:rastrigin]
    const N_GENES: usize = 2;
    const A: f32 = 10.0;

    fn rastrigin_function(genotype: Vec<f32>) -> f32 {
        let mut value = A * N_GENES as f32;
        for i in 0..N_GENES {
            value += genotype[i].powi(2) - A * (2.0 * std::f32::consts::PI * genotype[i]).cos();
        }

        value
    }

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(N_GENES, -5.12..5.12))
        .minimizing()
        .fitness_fn(rastrigin_function)
        .build();
    // --8<-- [end:rastrigin]

    // --8<-- [start:batch]
    let engine = GeneticEngine::builder()
        .codec(IntChromosome::from((5, 0..100)))
        // Replace the original 'fitness_fn' call with 'batch_fitness_fn' to enable batch fitness.
        // This fitness function will receive a single batch containing all individuals which need evaluation
        .batch_fitness_fn(|phenotypes: &[Vec<i32>]| {
            phenotypes
                .iter()
                .map(|geno| geno.iter().sum::<i32>())
                .collect()
        })
        .build();

    // When using an parallel executor, the batches will be grouped together to evaluate on separate threads
    let engine = GeneticEngine::builder()
        .codec(IntChromosome::from((5, 0..100)))
        // 'batch_fitness_fn' will receive 7 batches of individuals during each generation's evaluation
        .executor(Executor::FixedSizedWorkerPool(7))
        .batch_fitness_fn(|phenotypes: &[Vec<i32>]| {
            phenotypes
                .iter()
                .map(|geno| geno.iter().sum::<i32>())
                .collect()
        })
        .build();
    // --8<-- [end:batch]

    // --8<-- [start:raw]
    fn my_fitness_fn(genotype: &Genotype<FloatChromosome<f32>>) -> f32 {
        // Evaluate the genotype directly without decoding
        genotype
            .iter()
            .map(|chromosome| chromosome.iter().map(|gene| *gene.allele()).sum::<f32>())
            .sum::<f32>()
    }

    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, 0.0..1.0))
        .raw_fitness_fn(my_fitness_fn)
        // or .raw_batch_fitness_fn(...) for batch raw fitness functions
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|epoch| epoch.index() >= 100);
    // --8<-- [end:raw]

    let my_model_codec = FloatCodec::vector(10, 0.0..1.0); // decodes to MyDecodedModel (Vec<f32>)

    // --8<-- [start:composite]
    fn accuracy_objective(model: &MyDecodedModel) -> f32 {
        // ... calculate accuracy ...
        model.iter().sum()
    }

    fn complexity_objective(model: &MyDecodedModel) -> f32 {
        // ... calculate complexity ...
        model.len() as f32
    }

    fn efficiency_objective(model: &MyDecodedModel) -> f32 {
        // ... calculate efficiency ...
        model.iter().copied().fold(0.0, f32::max)
    }

    // Create weighted composite fitness function - this version computes a
    // weighted average of each function given to it
    let composite_fitness = CompositeFitnessFn::new()
        .add_weighted_fn(accuracy_objective, 0.6) // 60% weight on accuracy
        .add_weighted_fn(complexity_objective, 0.25) // 25% weight on complexity
        .add_weighted_fn(efficiency_objective, 0.15); // 15% weight on efficiency

    // Create an equal weight composite fitness function with equal weights.
    // Meaning these are all essentially just added together to create a single objective.
    let composite_fitness = CompositeFitnessFn::new()
        .add_fitness_fn(accuracy_objective)
        .add_fitness_fn(complexity_objective)
        .add_fitness_fn(efficiency_objective);

    // Add it to the engine like any other fitness function
    let engine = GeneticEngine::builder()
        .codec(my_model_codec)
        .fitness_fn(composite_fitness)
        .build();
    // --8<-- [end:composite]

    // A codec whose decoded type is `MyModel` so the novelty descriptor below can read it.
    let my_model_codec = FnCodec::new()
        .with_encoder(|| Genotype::from(FloatChromosome::from((10, 0.0..1.0))))
        .with_decoder(|geno: &Genotype<FloatChromosome<f32>>| {
            MyModel(geno[0].iter().map(|gene| *gene.allele()).collect())
        });

    // --8<-- [start:novelty]
    // Define a behavioral descriptor
    struct MyModelBehaviorDescriptor;

    // ... rest of impl ...

    impl Novelty<MyModel> for MyModelBehaviorDescriptor {
        fn description(&self, individual: &MyModel) -> Vec<f32> {
            // Return behavioral characteristics (e.g., outputs on test cases)
            individual.get_behavior_vector()
        }
    }

    // Create novelty search fitness function
    let novelty_fitness = NoveltySearch::new(MyModelBehaviorDescriptor)
        .k(10)
        .threshold(0.1)
        .archive_size(1000) // Optional: set archive size - default is 1000
        .cosine_distance(); // Optional set the distance parameter used
    // .euclidean_distance() // euclidean_distance is the default
    // .hamming_distance()

    // Novelty is also implemented for any F where F: Fn(&T) -> Vec<f32>. Meaning, you can
    // just as easily feed a function to NoveltySearch as long as it takes a borrowed T (&T)
    // and returns a Vec<f32>
    let function_novelty_fitness =
        NoveltySearch::new(|individual: &MyModel| individual.get_behavior_vector());

    let engine = GeneticEngine::builder()
        // The decoded genotype from your codec (my_model_codec in this case) will be fed
        // into the `description` function from the Novelty trait impl
        .codec(my_model_codec)
        .maximizing()
        .fitness_fn(novelty_fitness)
        .build();
    // --8<-- [end:novelty]
}
