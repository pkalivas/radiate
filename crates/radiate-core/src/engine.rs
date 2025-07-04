pub trait Engine {
    type Epoch;
    fn next(&mut self) -> Self::Epoch;
}

pub trait EngineExt<E: Engine> {
    fn run<F>(&mut self, limit: F) -> E::Epoch
    where
        F: Fn(&E::Epoch) -> bool;
}

impl<E> EngineExt<E> for E
where
    E: Engine,
{
    fn run<F>(&mut self, limit: F) -> E::Epoch
    where
        F: Fn(&E::Epoch) -> bool,
    {
        loop {
            let epoch = self.next();

            if limit(&epoch) {
                break epoch;
            }
        }
    }
}
