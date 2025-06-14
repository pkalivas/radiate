#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    data: Vec<f32>,
    length: usize,
}

impl Polygon {
    pub fn new(length: usize) -> Self {
        Self {
            data: vec![0.0; 4 + 2 * length],
            length,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }
}
