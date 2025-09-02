use core::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct Pred<G> {
    pub(crate) f: Arc<dyn Fn(&G) -> bool + Send + Sync + 'static>,
    name: Option<&'static str>,
}

impl<G> Pred<G> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&G) -> bool + Send + Sync + 'static,
    {
        Self {
            f: Arc::new(f),
            name: None,
        }
    }

    pub fn named<F>(name: &'static str, f: F) -> Self
    where
        F: Fn(&G) -> bool + Send + Sync + 'static,
    {
        Self {
            f: Arc::new(f),
            name: Some(name),
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    #[inline]
    pub fn test(&self, g: &G) -> bool {
        (self.f)(g)
    }
}

impl<G> fmt::Debug for Pred<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.name {
            write!(f, "Pred({n})")
        } else {
            write!(f, "Pred(<closure>)")
        }
    }
}
