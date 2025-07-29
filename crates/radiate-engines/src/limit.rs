use radiate_core::Score;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Limit {
    Generation(usize),
    Seconds(Duration),
    Score(Score),
    Convergence(usize, f32),
    Combined(Vec<Limit>),
}

impl Into<Limit> for usize {
    fn into(self) -> Limit {
        Limit::Generation(self)
    }
}

impl Into<Limit> for Duration {
    fn into(self) -> Limit {
        Limit::Seconds(self)
    }
}

impl Into<Limit> for f32 {
    fn into(self) -> Limit {
        Limit::Score(Score::from(self))
    }
}

impl Into<Limit> for Vec<f32> {
    fn into(self) -> Limit {
        Limit::Score(Score::from(self))
    }
}

impl Into<Limit> for (usize, f32) {
    fn into(self) -> Limit {
        Limit::Convergence(self.0, self.1)
    }
}

impl Into<Limit> for Vec<Limit> {
    fn into(self) -> Limit {
        Limit::Combined(self)
    }
}
