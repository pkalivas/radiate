use crate::Objective;

#[derive(Debug, Clone)]
pub struct Tracker<T>
where
    T: Clone + PartialOrd,
{
    pub best: Option<T>,
    pub current: Option<T>,
    pub stagnation: usize,
}

impl<T> Tracker<T>
where
    T: Clone + PartialOrd,
{
    pub fn new() -> Self {
        Tracker {
            best: None,
            current: None,
            stagnation: 0,
        }
    }

    pub fn update(&mut self, other: &T, optimize: &Objective) {
        self.current = Some(other.clone());

        if let Some(best) = &self.best {
            if optimize.is_better(other, best) {
                self.best = Some(other.clone());
                self.stagnation = 0;
            } else {
                self.stagnation += 1;
            }
        } else {
            self.best = Some(other.clone());
            self.stagnation = 0;
        }
    }

    pub fn stagnation(&self) -> usize {
        self.stagnation
    }

    pub fn best(&self) -> Option<&T> {
        self.best.as_ref()
    }

    pub fn current(&self) -> Option<&T> {
        self.current.as_ref()
    }
}
