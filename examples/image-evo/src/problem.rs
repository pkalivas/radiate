use crate::{
    chromosome::{ImageChromosome, ImageGene},
    polygon::Polygon,
};
use image::{DynamicImage, ImageBuffer, Rgba};
use radiate::*;

pub struct ImageProblem {
    target_pixels: Vec<Rgba<u8>>,
    codec: FnCodec<ImageChromosome, ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

impl ImageProblem {
    pub fn new(num_genes: usize, polygon_size: usize, image_path: DynamicImage) -> Self {
        let target_image = image_path.to_rgba8();
        let image_height = target_image.height();
        let image_width = target_image.width();

        Self {
            target_pixels: target_image.pixels().cloned().collect(),
            codec: FnCodec::new()
                .with_encoder(move || {
                    Genotype::from(ImageChromosome::new(
                        (0..num_genes)
                            .map(|_| ImageGene::new(Polygon::new_random(polygon_size)))
                            .collect(),
                    ))
                })
                .with_decoder(move |genotype: &Genotype<ImageChromosome>| {
                    genotype[0].draw(image_width, image_height)
                }),
        }
    }
}

impl Problem<ImageChromosome, ImageBuffer<Rgba<u8>, Vec<u8>>> for ImageProblem {
    fn encode(&self) -> Genotype<ImageChromosome> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<ImageChromosome>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<ImageChromosome>) -> Score {
        let decoded = self.decode(individual);

        let mut diff = 0.0;

        for (p1, p2) in decoded.pixels().zip(self.target_pixels.iter()) {
            let dr = p2[0] as f32 - p1[0] as f32;
            let dg = p2[1] as f32 - p1[1] as f32;
            let db = p2[2] as f32 - p1[2] as f32;

            diff += dr * dr + dg * dg + db * db;
        }

        let score = diff / (decoded.width() * decoded.height() * 3) as f32;

        Score::from(score)
    }
}

unsafe impl Send for ImageProblem {}
unsafe impl Sync for ImageProblem {}
