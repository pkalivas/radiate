mod alters;
mod diversity;
mod executors;
mod limits;
mod selectors;

pub trait InputConverter<O> {
    fn convert(&self) -> O;
}
