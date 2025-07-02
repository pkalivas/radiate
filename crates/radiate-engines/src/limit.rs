#[derive(Debug, Clone)]
pub enum Limit {
    Generation(usize),
    Seconds(f64),
    Score(f32),
}
