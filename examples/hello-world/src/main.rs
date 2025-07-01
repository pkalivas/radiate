use radiate::*;

fn main() {
    random_provider::set_seed(100);

    let target = "Hello, Radiate!";

    let engine = GeneticEngine::builder()
        .codec(CharCodec::vector(target.len()))
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

    engine
        .iter()
        .until_score(target.len())
        .inspect(|generation| println!("{:?}", generation))
        .take(1)
        .last()
        .unwrap();
}
