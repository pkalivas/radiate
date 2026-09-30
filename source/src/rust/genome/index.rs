use radiate::prelude::*;
use std::sync::Arc;

fn main() {
    // --8<-- [start:float_gene]
    // Create a float gene that can evolve between -1.0 and 1.0 but
    // must stay within -10.0 to 10.0 during evolution
    let gene: FloatGene<f32> = FloatGene::new(0.5, -1.0..1.0, -10.0..10.0);
    let gene_f64: FloatGene<f64> = FloatGene::new(0.5, -1_f64..1_f64, -10_f64..10_f64);

    // Create a float gene with a randomly generated allele between -1.0 and 1.0
    // and bounds between -1.0 and 1.0
    let gene = FloatGene::from(-1.0..1.0);

    // Create a float gene with a randomly generated allele between -1.0 and 1.0 with bounds between -10.0 and 10.0
    let gene = FloatGene::from((-1.0..1.0, -10.0..10.0));

    // Create a float gene with an allele of 0.5 allele between -1.0 and 1.0 with bounds between -10.0 and 10.0
    let gene = FloatGene::new(0.5, -1.0..1.0, -10.0..10.0);
    // --8<-- [end:float_gene]

    // --8<-- [start:int_gene]
    // Create an integer gene that can evolve between -100 and 100
    let gene = IntGene::new(42, -10..10, -100..100);

    // Create an integer gene with a randomly generated allele between -10 and 10 - specify the int type
    let gene = IntGene::<i8>::from(-10..10);

    // Create an integer gene with a randomly generated allele between -10 and 10 with bounds between -100 and 100
    let gene = IntGene::from((-10..10, -100..100));

    // Create an integer gene with an allele of 42 between -10 and 10 with bounds between -100 and 100
    let gene = IntGene::new(42, -10..10, -100..100);
    // --8<-- [end:int_gene]

    // --8<-- [start:bit_gene]
    // Create an bit gene with a randomly generated allele of true or false.
    let gene = BitGene::new();

    // Create a bit gene with an allele of true
    let gene = BitGene::from(true);
    // --8<-- [end:bit_gene]

    // --8<-- [start:char_gene]
    // Create an char gene with a randomly generated allele from the ASCII printable characters
    let gene = CharGene::default();

    // Create a char gene with a char_set of 'abc' of which the allele will be randomly chosen from
    let gene = CharGene::from("abc");

    // Create a char gene with an allele of 'A' from the char_set 'abc'
    let gene = CharGene::new(Arc::new(['a', 'b', 'c']));
    // --8<-- [end:char_gene]

    // --8<-- [start:permutation_gene]
    // Define a list of alleles the associated genes
    let alleles: Arc<[i32]> = vec![1, 2, 3, 4].into_boxed_slice().into();
    let genes = vec![
        PermutationGene::new(0, Arc::clone(&alleles)),
        PermutationGene::new(1, Arc::clone(&alleles)),
        PermutationGene::new(2, Arc::clone(&alleles)),
        PermutationGene::new(3, Arc::clone(&alleles)),
    ];
    // --8<-- [end:permutation_gene]

    // --8<-- [start:float_chromosome]
    // Create a float chromosome with 5 genes, each initialized to a random value between -1.0 and 1.0
    let chromosome = FloatChromosome::from((5, -1.0..1.0));
    let f64_chromosome: FloatChromosome<f64> = FloatChromosome::from((5, -1_f64..1_f64));

    let bounded_chromosome = FloatChromosome::from((5, -1.0..1.0, -10.0..10.0));
    // --8<-- [end:float_chromosome]

    // --8<-- [start:int_chromosome]
    // Create an integer chromosome with 5 genes, each initialized to a random value between -10 and 10
    let chromosome = IntChromosome::<i32>::from((5, -10..10));

    let bounded_chromosome = IntChromosome::<i32>::from((5, -10..10, -100..100));
    // --8<-- [end:int_chromosome]

    // --8<-- [start:bit_chromosome]
    // Create a bit chromosome with 5 genes, each initialized to a random value of true or false
    let chromosome = BitChromosome::new(5);
    // --8<-- [end:bit_chromosome]

    // --8<-- [start:char_chromosome]
    // Create a character chromosome with 5 genes, each initialized to
    // a random character from the provided char_set
    let chromosome = CharChromosome::from((5, Some("abc")));
    // --8<-- [end:char_chromosome]

    // --8<-- [start:permutation_chromosome]
    let alleles = Arc::new(vec![1, 2, 3, 4]);
    // FIXME
    // let chromosome = PermutationChromosome::from((4, Arc::clone(&alleles)));
    // --8<-- [end:permutation_chromosome]

    // --8<-- [start:genotype]
    // Create a genotype with a single FloatChromosome and a 5 FloatGenes
    let genotype = Genotype::from(FloatChromosome::from((5, -1.0..1.0)));
    // -- or --
    let genotype = Genotype::from(FloatChromosome::from(FloatGene::new(
        0.1,
        -1.0..1.0,
        -10.0..10.0,
    )));

    // Create a genotype with multiple chromosomes of lengths 5, 15, and 3
    let mut three_chromosome_genotype = Genotype::new(vec![
        FloatChromosome::from((5, -1.0..1.0)),
        FloatChromosome::from((15, -1.0..1.0)),
        FloatChromosome::from((3, -1.0..1.0)),
    ]);

    let genotype_length = three_chromosome_genotype.len(); // 3

    // Get the second chromosome from the genotype
    let second_chromosome = three_chromosome_genotype.get(1).unwrap(); // or use `three_chromosome_genotype[1]`
    let second_chromosome_mut = three_chromosome_genotype.get_mut(1).unwrap();

    for chromosome in three_chromosome_genotype.iter() { // or iter_mut()
        // Do something with each chromosome
    }
    // --8<-- [end:genotype]
}
