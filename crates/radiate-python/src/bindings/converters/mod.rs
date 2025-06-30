mod alters;
mod diversity;
mod selectors;

pub trait InputConverter<O> {
    fn convert(&self) -> O;
}
