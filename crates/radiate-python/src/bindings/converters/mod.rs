mod alters;
mod diversity;
mod executors;
mod limits;
mod selectors;

pub trait InputTransform<O> {
    fn transform(&self) -> O;
}
