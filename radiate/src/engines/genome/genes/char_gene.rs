use super::gene::{Gene, Valid};

const ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"$%&/()=?`{[]}\\+~*#';.:,-_<>|@^' ";

pub struct CharGene {
    pub allele: char,
}

impl CharGene {
    pub fn new() -> Self {
        let index = rand::random::<usize>() % ALPHABET.len();
        Self {
            allele: ALPHABET.chars().nth(index).unwrap(),
        }
    }
}

impl Gene<CharGene, char> for CharGene {
    fn allele(&self) -> &char {
        &self.allele
    }

    fn new_instance(&self) -> CharGene {
        CharGene::new()
    }

    fn from_allele(&self, allele: &char) -> CharGene {
        CharGene { allele: *allele }
    }
}

impl Valid for CharGene {}

impl Clone for CharGene {
    fn clone(&self) -> Self {
        CharGene {
            allele: self.allele,
        }
    }
}

impl PartialEq for CharGene {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl std::fmt::Debug for CharGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}
