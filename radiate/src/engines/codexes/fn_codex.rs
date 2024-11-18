use crate::{Gene, Genotype};

use super::Codex;


pub struct FnCodex<G, A, T>
where
    G: Gene<G, A>,
{
    pub encode_fn: Option<fn() -> Genotype<G, A>>,
    pub decode_fn: Option<fn(&Genotype<G, A>) -> T>,
}

impl<G, A, T> FnCodex<G, A, T>
where
    G: Gene<G, A>,
{
    pub fn new() -> Self {
        Self {
            encode_fn: None,
            decode_fn: None,
        }
    }

    pub fn encoder(mut self, encode_fn: fn() -> Genotype<G, A>) -> Self {
        self.encode_fn = Some(encode_fn);
        self
    }

    pub fn decoder(mut self, decode_fn: fn(&Genotype<G, A>) -> T) -> Self {
        self.decode_fn = Some(decode_fn);
        self
    }
}

impl<G, A, T> Codex<G, A, T> for FnCodex<G, A, T>
where
    G: Gene<G, A>,
{
    fn encode(&self) -> Genotype<G, A> {
        match self.encode_fn {
            Some(encode_fn) => encode_fn(),
            None => panic!("No encode function provided"),
        }
    }

    fn decode(&self, genotype: &Genotype<G, A>) -> T {
        match self.decode_fn {
            Some(decode_fn) => decode_fn(genotype),
            None => panic!("No decode function provided"),
        }
    }
}