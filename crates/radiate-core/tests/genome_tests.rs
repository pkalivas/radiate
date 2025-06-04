#[cfg(test)]
mod serde_tests {

    #[allow(unused_imports)]
    use radiate_core::genome::chromosomes::bit::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_bit_gene_serialization() {
        let gene = BitGene::new();
        let serialized = serde_json::to_string(&gene).expect("Failed to serialize BitGene");
        let deserialized: BitGene =
            serde_json::from_str(&serialized).expect("Failed to deserialize BitGene");

        println!("Serialized BitGene: {}", serialized);
        assert_eq!(gene, deserialized);
    }
}
