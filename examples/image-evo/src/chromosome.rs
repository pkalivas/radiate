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
            allele: Polygon::new(self.allele.len()),
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
            allele: Polygon::new(self.allele.len()),
        }
    }
}

impl Add for ImageGene {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] = (self.allele[0] + other.allele[0]).clamp(0.0, 1.0); // r
        allele[1] = (self.allele[1] + other.allele[1]).clamp(0.0, 1.0); // g
        allele[2] = (self.allele[2] + other.allele[2]).clamp(0.0, 1.0); // b
        allele[3] = (self.allele[3] + other.allele[3]).clamp(0.0, 1.0); // a

        // Position addition (average positions)
        for i in 4..allele.len() {
            allele[i] = (self.allele[i] + other.allele[i]) * 0.5;
        }

        Self { allele }
    }
}

impl Sub for ImageGene {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] = (self.allele[0] - other.allele[0]).clamp(0.0, 1.0); // r
        allele[1] = (self.allele[1] - other.allele[1]).clamp(0.0, 1.0); // g
        allele[2] = (self.allele[2] - other.allele[2]).clamp(0.0, 1.0); // b
        allele[3] = (self.allele[3] - other.allele[3] * 0.5).clamp(0.0, 1.0); // a

        // Position subtraction (move away from other polygon)
        for i in 4..allele.len() {
            let diff = self.allele[i] - other.allele[i];
            allele[i] = (self.allele[i] + diff * 0.1).clamp(0.0, 1.0);
        }

        Self { allele }
    }
}

impl Mul for ImageGene {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        // Color multiplication (component-wise)
        allele[0] = (self.allele[0] * other.allele[0]).clamp(0.0, 1.0); // r
        allele[1] = (self.allele[1] * other.allele[1]).clamp(0.0, 1.0); // g
        allele[2] = (self.allele[2] * other.allele[2]).clamp(0.0, 1.0); // b
        allele[3] = (self.allele[3] * other.allele[3]).clamp(0.0, 1.0); // a

        // Position multiplication (scale positions relative to center)
        let center_x =
            allele.iter().skip(4).step_by(2).sum::<f32>() / (allele.len() - 4) as f32 * 0.5;
        let center_y =
            allele.iter().skip(5).step_by(2).sum::<f32>() / (allele.len() - 4) as f32 * 0.5;

        for i in (4..allele.len()).step_by(2) {
            let rel_x = allele[i] - center_x;
            allele[i] = (center_x + rel_x * other.allele[0]).clamp(0.0, 1.0);
        }
        for i in (5..allele.len()).step_by(2) {
            let rel_y = allele[i] - center_y;
            allele[i] = (center_y + rel_y * other.allele[1]).clamp(0.0, 1.0);
        }

        Self { allele }
    }
}

impl Div for ImageGene {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        let mut allele = self.allele.clone();

        allele[0] = (self.allele[0] / other.allele[0].max(0.001)).clamp(0.0, 1.0); // r
        allele[1] = (self.allele[1] / other.allele[1].max(0.001)).clamp(0.0, 1.0); // g
        allele[2] = (self.allele[2] / other.allele[2].max(0.001)).clamp(0.0, 1.0); // b
        allele[3] = (self.allele[3] / other.allele[3].max(0.001)).clamp(0.0, 1.0); // a

        // Position division (inverse scaling)
        let center_x =
            allele.iter().skip(4).step_by(2).sum::<f32>() / (allele.len() - 4) as f32 * 0.5;
        let center_y =
            allele.iter().skip(5).step_by(2).sum::<f32>() / (allele.len() - 4) as f32 * 0.5;

        for i in (4..allele.len()).step_by(2) {
            let rel_x = allele[i] - center_x;
            let scale = other.allele[0].max(0.001);
            allele[i] = (center_x + rel_x / scale).clamp(0.0, 1.0);
        }
        for i in (5..allele.len()).step_by(2) {
            let rel_y = allele[i] - center_y;
            let scale = other.allele[1].max(0.001);
            allele[i] = (center_y + rel_y / scale).clamp(0.0, 1.0);
        }

        Self { allele }
    }
}

impl From<Polygon> for ImageGene {
    fn from(allele: Polygon) -> Self {
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
