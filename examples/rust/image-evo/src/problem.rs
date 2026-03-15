use crate::{
    chromosome::{ImageChromosome, ImageGene},
    polygon::Polygon,
};
use image::{DynamicImage, ImageBuffer, Rgba};
use radiate::prelude::*;

#[derive(Debug, Clone)]
pub struct ImageProblem {
    target_pixels: Vec<Rgba<u8>>,
    num_genes: usize,
    polygon_size: usize,
    width: u32,
    height: u32,
}

impl ImageProblem {
    pub fn new(num_genes: usize, polygon_size: usize, image_path: DynamicImage) -> Self {
        let target_image = image_path.to_rgba8();
        let (image_width, image_height) = target_image.dimensions();

        Self {
            target_pixels: target_image.pixels().cloned().collect(),
            num_genes,
            polygon_size,
            width: image_width,
            height: image_height,
        }
    }
}

impl Problem<ImageChromosome, ImageBuffer<Rgba<u8>, Vec<u8>>> for ImageProblem {
    fn encode(&self) -> Genotype<ImageChromosome> {
        Genotype::from(ImageChromosome::new(
            (0..self.num_genes)
                .map(|_| ImageGene::new(Polygon::new(self.polygon_size)))
                .collect(),
        ))
    }

    fn decode(&self, genotype: &Genotype<ImageChromosome>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        genotype[0].draw(self.width, self.height)
    }

    fn eval(&self, individual: &Genotype<ImageChromosome>) -> radiate::RadiateResult<Score> {
        let decoded = self.decode(individual);

        let mut diff = 0.0;
        for (p1, p2) in decoded.pixels().zip(self.target_pixels.iter()) {
            let dr = p2[0] as f32 - p1[0] as f32;
            let dg = p2[1] as f32 - p1[1] as f32;
            let db = p2[2] as f32 - p1[2] as f32;

            diff += dr * dr + dg * dg + db * db;
        }

        let score = diff / (self.target_pixels.len() * 3) as f32;

        Ok(Score::from(score.sqrt()))
    }
}

unsafe impl Send for ImageProblem {}
unsafe impl Sync for ImageProblem {}
