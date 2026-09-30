use crate::{Chromosome, Codec, Genotype, PackedBitChromosome};

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
        let tail_mask = genotype[0].tail_mask();
        let mut words = genotype[0].iter().map(|w| w.get()).collect::<Vec<_>>();
        if let Some(last) = words.last_mut() {
            *last &= tail_mask;
        }

        words
    }
}
