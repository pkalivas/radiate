use crate::{BitChromosome, Chromosome, ContiguousChromosome, Gene, Valid, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const WORD_BITS: usize = 64;

/// 64 bits of a [`PackedBitChromosome`]. The gene is the word, not the bit, so
/// `&mut BitWord` is a real reference and the chromosome can implement
/// [`ContiguousChromosome`] without proxies.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct BitWordGene(u64);

impl BitWordGene {
    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn get_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Valid for BitWordGene {}

impl Gene for BitWordGene {
    type Allele = u64;

    fn allele(&self) -> &u64 {
        &self.0
    }

    fn allele_mut(&mut self) -> &mut u64 {
        &mut self.0
    }

    fn new_instance(&self) -> Self {
        BitWordGene(random_provider::random())
    }

    fn with_allele(&self, allele: &u64) -> Self {
        BitWordGene(*allele)
    }

    fn set_allele(&mut self, allele: u64) {
        self.0 = allele;
    }
}

/// A bit string stored 64 bits per [`BitWord`] gene.
///
/// Generic operators see 64-bit words. Bit-level access goes through the methods
/// below. Bits at positions `>= num_bits` in the last word are unspecified, and
/// every bit-level reader masks them.
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PackedBitChromosome {
    words: Vec<BitWordGene>,
    num_bits: usize,
}

impl PackedBitChromosome {
    pub fn new(num_bits: usize) -> Self {
        let words = (0..num_words(num_bits))
            .map(|_| BitWordGene(random_provider::random()))
            .collect();
        let mut chromosome = PackedBitChromosome { words, num_bits };
        chromosome.clear_tail();
        chromosome
    }

    /// An all-zero bit string of `num_bits`. Draws nothing from the RNG.
    fn zeros(num_bits: usize) -> Self {
        PackedBitChromosome {
            words: vec![BitWordGene(0); num_words(num_bits)],
            num_bits,
        }
    }

    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    pub fn bit(&self, i: usize) -> bool {
        (self.words[i / WORD_BITS].0 >> (i % WORD_BITS)) & 1 != 0
    }

    pub fn set_bit(&mut self, i: usize, value: bool) {
        let mask = 1u64 << (i % WORD_BITS);
        let word = &mut self.words[i / WORD_BITS].0;
        if value {
            *word |= mask;
        } else {
            *word &= !mask;
        }
    }

    pub fn flip_bit(&mut self, i: usize) {
        self.words[i / WORD_BITS].0 ^= 1u64 << (i % WORD_BITS);
    }

    /// Tail-masked popcount.
    pub fn count_ones(&self) -> usize {
        match self.words.split_last() {
            Some((last, rest)) => {
                rest.iter()
                    .map(|w| w.0.count_ones() as usize)
                    .sum::<usize>()
                    + (last.0 & self.tail_mask()).count_ones() as usize
            }
            None => 0,
        }
    }

    pub fn iter_bits(&self) -> impl Iterator<Item = bool> + '_ {
        (0..self.num_bits).map(move |i| self.bit(i))
    }

    /// Zero the unspecified bits past `num_bits`.
    pub fn clear_tail(&mut self) {
        let mask = self.tail_mask();
        if let Some(last) = self.words.last_mut() {
            last.0 &= mask;
        }
    }

    /// Mask of the valid bits in the last word (`u64::MAX` when `num_bits % 64 == 0`).
    pub fn tail_mask(&self) -> u64 {
        let tail_bits = self.num_bits % WORD_BITS;
        if tail_bits == 0 {
            u64::MAX
        } else {
            (1u64 << tail_bits) - 1
        }
    }
}

fn num_words(num_bits: usize) -> usize {
    num_bits.div_ceil(WORD_BITS)
}

impl FromIterator<bool> for PackedBitChromosome {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        let mut words = Vec::new();
        let mut num_bits = 0;
        for bit in iter {
            let bit_index = num_bits % WORD_BITS;
            if bit_index == 0 {
                words.push(BitWordGene(0));
            }

            if bit {
                if let Some(last) = words.last_mut() {
                    last.0 |= 1u64 << bit_index;
                }
            }

            num_bits += 1;
        }

        PackedBitChromosome { words, num_bits }
    }
}

impl From<&BitChromosome> for PackedBitChromosome {
    fn from(bit_chromosome: &BitChromosome) -> Self {
        let mut chromosome = PackedBitChromosome::zeros(bit_chromosome.len());
        for (i, gene) in bit_chromosome.iter().enumerate() {
            if *gene.allele() {
                chromosome.words[i / WORD_BITS].0 |= 1u64 << (i % WORD_BITS);
            }
        }

        chromosome
    }
}

/// Equality ignores the unspecified tail bits.
impl PartialEq for PackedBitChromosome {
    fn eq(&self, other: &Self) -> bool {
        if self.num_bits != other.num_bits {
            return false;
        }

        let full_words = self.num_bits / WORD_BITS;
        if self.words[..full_words] != other.words[..full_words] {
            return false;
        }

        if !self.num_bits.is_multiple_of(WORD_BITS) {
            let mask = self.tail_mask();
            return (self.words[full_words].0 & mask) == (other.words[full_words].0 & mask);
        }

        true
    }
}

impl Valid for PackedBitChromosome {}

impl Chromosome for PackedBitChromosome {
    type Gene = BitWordGene;

    fn iter(&self) -> impl Iterator<Item = &Self::Gene> {
        self.words.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Gene> {
        self.words.iter_mut()
    }

    fn get(&self, index: usize) -> Option<&Self::Gene> {
        self.words.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        self.words.get_mut(index)
    }

    fn set(&mut self, index: usize, gene: Self::Gene) {
        if let Some(word) = self.words.get_mut(index) {
            *word = gene;
        }
    }

    /// Number of words (genes), not bits. See [`PackedBitChromosome::num_bits`].
    fn len(&self) -> usize {
        self.words.len()
    }
}

impl ContiguousChromosome for PackedBitChromosome {
    fn as_slice(&self) -> &[BitWordGene] {
        &self.words
    }

    fn as_mut_slice(&mut self) -> &mut [BitWordGene] {
        &mut self.words
    }
}
