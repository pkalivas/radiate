use image::{Rgba, RgbaImage};
use imageproc::point::Point;
use radiate::random_provider;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    data: Vec<f32>,
    length: usize,
}

impl Polygon {
    pub fn new(length: usize) -> Self {
        let mut result = Self::empty(length);

        result[0] = random_provider::random::<f32>(); // r
        result[1] = random_provider::random::<f32>(); // g
        result[2] = random_provider::random::<f32>(); // b

        let a_base = random_provider::random::<f32>() * random_provider::random::<f32>();
        result[3] = f32::max(0.2, a_base);

        let mut px = random_provider::random::<f32>();
        let mut py = random_provider::random::<f32>();

        for k in 0..length {
            px += random_provider::random::<f32>() - 0.5_f32;
            py += random_provider::random::<f32>() - 0.5_f32;

            px = px.clamp(0.0, 1.0);
            py = py.clamp(0.0, 1.0);

            result[4 + 2 * k] = px;
            result[5 + 2 * k] = py;
        }

        result
    }

    pub fn empty(length: usize) -> Self {
        Self {
            data: vec![0.0; 4 + 2 * length],
            length,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, f32> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn mean(&self, other: &Self) -> Self {
        let mut result = Self::empty(self.length);
        for i in 0..self.data.len() {
            result.data[i] = (self.data[i] + other.data[i]) * 0.5;
        }

        result
    }

    pub fn draw(&self, canvas: &mut RgbaImage) {
        let color = Rgba([
            (self[0] * 255.0) as u8,
            (self[1] * 255.0) as u8,
            (self[2] * 255.0) as u8,
            (self[3] * 255.0) as u8,
        ]);

        let width = canvas.width();
        let height = canvas.height();

        let mut points = Vec::with_capacity(self.length);
        for j in 0..self.length {
            let x = (self[4 + j * 2] * width as f32) as i32;
            let y = (self[5 + j * 2] * height as f32) as i32;
            points.push(Point::new(x, y));
        }

        points.dedup();

        if points.first() == points.last() {
            points.pop();
        }

        if points.len() < 3 {
            return;
        }

        let (min_x, max_x) = points
            .iter()
            .map(|p| p.x)
            .fold((i32::MAX, i32::MIN), |(min, max), x| {
                (min.min(x), max.max(x))
            });
        let (min_y, max_y) = points
            .iter()
            .map(|p| p.y)
            .fold((i32::MAX, i32::MIN), |(min, max), y| {
                (min.min(y), max.max(y))
            });

        let mask_width = (max_x - min_x + 1) as u32;
        let mask_height = (max_y - min_y + 1) as u32;

        if mask_width == 0 || mask_height == 0 {
            return;
        }

        let mut mask = RgbaImage::new(mask_width, mask_height);
        let white = Rgba([255, 255, 255, 255]);

        let adjusted_points: Vec<Point<i32>> = points
            .iter()
            .map(|p| Point::new(p.x - min_x, p.y - min_y))
            .collect();

        imageproc::drawing::draw_polygon_mut(&mut mask, &adjusted_points, white);

        for y in 0..mask_height {
            for x in 0..mask_width {
                let mask_pixel = mask.get_pixel(x, y);
                if mask_pixel[0] > 0 {
                    let canvas_x = (min_x as u32 + x).min(width - 1);
                    let canvas_y = (min_y as u32 + y).min(height - 1);

                    let existing = canvas.get_pixel(canvas_x, canvas_y);
                    let alpha = color[3] as f32 / 255.0;
                    let inverse = 1.0 - alpha;

                    let new_r = (color[0] as f32 * alpha + existing[0] as f32 * inverse) as u8;
                    let new_g = (color[1] as f32 * alpha + existing[1] as f32 * inverse) as u8;
                    let new_b = (color[2] as f32 * alpha + existing[2] as f32 * inverse) as u8;

                    canvas.put_pixel(canvas_x, canvas_y, Rgba([new_r, new_g, new_b, 255]));
                }
            }
        }
    }
}

impl Index<usize> for Polygon {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Polygon {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_new() {
        let polygon = Polygon::empty(3);
        assert_eq!(polygon.len(), 3);
        assert_eq!(polygon.data.len(), 10); // 4 + 2 * length
    }

    #[test]
    fn test_polygon_new_random() {
        let polygon = Polygon::new(3);
        assert_eq!(polygon.len(), 3);
        assert_eq!(polygon.data.len(), 10); // 4 + 2 * length
        assert!(polygon[0] >= 0.0 && polygon[0] <= 1.0); // r
        assert!(polygon[1] >= 0.0 && polygon[1] <= 1.0); // g
        assert!(polygon[2] >= 0.0 && polygon[2] <= 1.0); // b
        assert!(polygon[3] >= 0.2 && polygon[3] <= 1.0); // a
        for k in 0..3 {
            assert!(polygon[4 + 2 * k] >= 0.0 && polygon[4 + 2 * k] <= 1.0); // x
            assert!(polygon[5 + 2 * k] >= 0.0 && polygon[5 + 2 * k] <= 1.0); // y
        }
    }
}

// // circles with position, radius, and color
// pub struct Circle {
//     data: Vec<f32>, // [r, g, b, a, x, y, radius]
// }

// pub fn new_random() -> Self {
//     let mut result = Self::new();

//     result[0] = random_provider::random::<f32>(); // r
//     result[1] = random_provider::random::<f32>(); // g
//     result[2] = random_provider::random::<f32>(); // b
//     result[3] = 0.1 + random_provider::random::<f32>() * 0.9; // a
//     result[4] = random_provider::random::<f32>(); // x
//     result[5] = random_provider::random::<f32>(); // y
//     result[6] = 0.01 + random_provider::random::<f32>() * 0.2; // radius

//     result
// }
