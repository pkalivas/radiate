use super::polygon::*;
use image::{Rgba, RgbaImage};
use radiate::*;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, PartialEq)]
pub struct ImageGene {
    allele: Polygon,
}

impl ImageGene {
    pub fn new(allele: Polygon) -> Self {
        Self { allele }
    }

    pub fn draw(&self, img: &mut RgbaImage) {
        self.allele.draw(img);
    }
}

impl Valid for ImageGene {
    fn is_valid(&self) -> bool {
        self.allele.len() > 0
    }
}

impl Gene for ImageGene {
    type Allele = Polygon;

    fn allele(&self) -> &Self::Allele {
        &self.allele
    }

    fn new_instance(&self) -> Self {
        Self {
            allele: Polygon::new_random(self.allele.len()),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        Self {
            allele: allele.clone(),
        }
    }
}

impl ArithmeticGene for ImageGene {
    fn max(&self) -> &Self::Allele {
        &self.allele
    }

    fn min(&self) -> &Self::Allele {
        &self.allele
    }

    fn mean(&self, other: &Self) -> Self {
        let allele = self.allele.mean(&other.allele);
        Self { allele }
    }

    fn from_f32(&self, _: f32) -> Self {
        Self {
            allele: Polygon::new_random(self.allele.len()),
        }
    }
}

impl Add for ImageGene {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] = (self.allele[0] + other.allele[0]).min(1.0);
        allele[1] = (self.allele[1] + other.allele[1]).min(1.0);
        allele[2] = (self.allele[2] + other.allele[2]).min(1.0);

        Self { allele }
    }
}

impl Sub for ImageGene {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] = (self.allele[0] - other.allele[0]).max(0.0);
        allele[1] = (self.allele[1] - other.allele[1]).max(0.0);
        allele[2] = (self.allele[2] - other.allele[2]).max(0.0);

        Self { allele }
    }
}

impl Mul for ImageGene {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] *= other.allele[0];
        allele[1] *= other.allele[1];
        allele[2] *= other.allele[2];

        Self { allele }
    }
}

impl Div for ImageGene {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] /= other.allele[0].max(0.001);
        allele[1] /= other.allele[1].max(0.001);
        allele[2] /= other.allele[2].max(0.001);

        Self { allele }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageChromosome {
    genes: Vec<ImageGene>,
}

impl ImageChromosome {
    pub fn new(genes: Vec<ImageGene>) -> Self {
        Self { genes }
    }

    pub fn draw(&self, width: u32, height: u32) -> RgbaImage {
        let white = Rgba([255, 255, 255, 255]);
        let mut img = RgbaImage::from_pixel(width, height, white);

        for gene in self.genes.iter() {
            gene.draw(&mut img);
        }

        img
    }
}

impl Valid for ImageChromosome {
    fn is_valid(&self) -> bool {
        !self.genes.is_empty() && self.genes.iter().all(|g| g.is_valid())
    }
}

impl Chromosome for ImageChromosome {
    type Gene = ImageGene;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}
