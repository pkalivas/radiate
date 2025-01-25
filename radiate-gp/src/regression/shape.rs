#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    input_size: usize,
    output_size: usize,
}

impl Shape {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        Shape {
            input_size,
            output_size,
        }
    }

    pub fn input_size(&self) -> usize {
        self.input_size
    }

    pub fn output_size(&self) -> usize {
        self.output_size
    }
}

impl From<usize> for Shape {
    fn from(size: usize) -> Self {
        Shape::new(size, 1)
    }
}

impl From<(usize, usize)> for Shape {
    fn from(shape: (usize, usize)) -> Self {
        Shape::new(shape.0, shape.1)
    }
}
