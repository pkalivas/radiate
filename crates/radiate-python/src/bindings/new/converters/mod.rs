mod alters;
mod selectors;

use radiate::Chromosome;

pub trait InputConverter<C: Chromosome, O> {
    fn convert(&self) -> O;
}
