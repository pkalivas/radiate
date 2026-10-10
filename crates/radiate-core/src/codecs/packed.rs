use crate::{Codec, Genotype, PackedBitChromosome};

/// A [`Codec`] for a single [`PackedBitChromosome`] of `num_bits` bits.
///
/// Decodes to the chromosome's raw 64-bit words. Bit `i` is bit `i % 64` of word
/// `i / 64` (least-significant first), and the unused bits past `num_bits` in the
/// last word are cleared.
pub struct PackedBitCodec {
    num_bits: usize,
}

impl PackedBitCodec {
    pub fn new(num_bits: usize) -> Self {
        Self { num_bits }
    }
}

impl Codec<PackedBitChromosome, Vec<u64>> for PackedBitCodec {
    fn encode(&self) -> Genotype<PackedBitChromosome> {
        Genotype::from(PackedBitChromosome::new(self.num_bits))
    }

    fn decode(&self, genotype: &Genotype<PackedBitChromosome>) -> Vec<u64> {
        genotype[0].to_words()
    }
}
