/// A trait for types that can be created from a given input.
///
/// TODO: Document this trait.
pub trait Factory<T> {
    type Input;
    fn new_instance(&self, input: Self::Input) -> T;
}
