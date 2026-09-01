pub struct CheckpointActor {
    interval: usize,
}

impl CheckpointActor {
    pub fn new(interval: usize) -> Self {
        Self { interval }
    }
}
