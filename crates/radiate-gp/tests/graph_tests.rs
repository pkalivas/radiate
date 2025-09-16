#[cfg(test)]
mod tests {
    use radiate_gp::*;
    use std::io::Read;

    fn round(value: f32, places: u32) -> f32 {
        let factor = 10_f32.powi(places as i32);
        (value * factor).round() / factor
    }

    #[test]
    fn test_graph_from_json_is_valid_eval() {
        let file_path = std::env::current_dir()
            .unwrap()
            .join("tests/data/recurrent_graph.json");

        let file = std::fs::File::open(file_path).expect("Failed to open file");
        let mut buf = String::new();
        std::io::BufReader::new(file)
            .read_to_string(&mut buf)
            .expect("Failed to read recurrent_graph.json");

        let graph: Graph<Op<f32>> = serde_json::from_str(&buf).expect("Failed to parse JSON");

        let results = graph.eval(&vec![
            vec![0.0],
            vec![0.0],
            vec![0.0],
            vec![1.0],
            vec![0.0],
            vec![0.0],
            vec![0.0],
        ]);

        let expected = vec![
            vec![0.5],
            vec![0.0],
            vec![1.0],
            vec![0.0],
            vec![0.0],
            vec![0.0],
            vec![1.0],
        ];

        for (res, exp) in results.iter().zip(expected.iter()) {
            for (r, e) in res.iter().zip(exp.iter()) {
                let r = round(*r, 3);
                let e = round(*e, 3);
                assert!(
                    (r - e).abs() < 0.01,
                    "Expected {}, got {} (diff: {})",
                    e,
                    r,
                    (r - e).abs()
                );
            }
        }
    }
}
