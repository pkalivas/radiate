#[cfg(test)]
mod random_tests {

    use radiate::*;
    use std::io::BufRead;

    #[test]
    // #[ignore]
    fn random_seed_test() {
        seed_rng(42);

        let file_path = std::env::current_dir()
            .unwrap()
            .join("tests/data/random_values.csv");

        let file = std::fs::File::open(file_path).expect("Failed to open file");
        let reader = std::io::BufReader::new(file);

        let values_from_file: Vec<f32> = reader
            .lines()
            .map(|line| {
                line.expect("Failed to read line")
                    .parse::<f32>()
                    .expect("Failed to parse value")
            })
            .collect();

        for value in values_from_file {
            let random_value = random::<f32>();
            assert_eq!(value, random_value);
        }
    }
}
