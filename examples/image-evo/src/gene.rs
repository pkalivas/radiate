use super::polygon::*;
use radiate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ImageGene {
    allele: Polygon,
}

impl Valid for ImageGene {
    fn is_valid(&self) -> bool {
        self.allele.length() > 0
    }
}

impl Gene for ImageGene {
    type Allele = Polygon;

    fn allele(&self) -> &Self::Allele {
        &self.allele
    }

    fn new_instance(&self) -> Self {
        Self {
            allele: Polygon::new(self.allele.length()),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        Self {
            allele: allele.clone(),
        }
    }
}
