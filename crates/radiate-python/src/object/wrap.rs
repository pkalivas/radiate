use std::fmt::Debug;

#[repr(transparent)]
pub struct Wrap<T>(pub T);

impl<T> AsRef<T> for Wrap<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> Clone for Wrap<T> {
    fn clone(&self) -> Self {
        Wrap(self.0.clone())
    }
}

impl<T> From<T> for Wrap<T> {
    fn from(t: T) -> Self {
        Wrap(t)
    }
}

impl<T: Debug> Debug for Wrap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Wrap").field(&self.0).finish()
    }
}
