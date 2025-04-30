#[cfg(test)]
mod random_tests {

    use radiate_core::random_provider;
    use std::io::BufRead;

    #[test]
    fn random_seed_test() {
        random_provider::set_seed(42);

        let file_path = std::env::current_dir()
            .unwrap()
            .join("tests/data/random_values.csv");

        let file = std::fs::File::open(file_path).expect("Failed to open file");

        let values_from_file = std::io::BufReader::new(file)
            .lines()
            .map(|line| {
                line.expect("Failed to read line")
                    .parse::<f32>()
                    .expect("Failed to parse value")
            })
            .collect::<Vec<f32>>();

        for value in values_from_file {
            let random_value = random_provider::random::<f32>();
            assert_eq!(value, random_value);
        }
    }
}
