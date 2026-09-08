use crate::AnyValue;

pub struct AnyList<'a, T>
where
    AnyValue<'a>: for<'b> From<&'b T>,
{
    values: &'a [T],
}

impl<'a, T> AnyList<'a, T>
where
    AnyValue<'a>: for<'b> From<&'b T>,
{
    pub fn new(values: &'a [T]) -> Self {
        Self { values }
    }

    pub fn values(&self) -> &'a [T] {
        self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.values.get(index)
    }
}

impl<'a, T> From<AnyList<'a, T>> for AnyValue<'a>
where
    AnyValue<'a>: for<'b> From<&'b T>,
{
    fn from(val: AnyList<'a, T>) -> Self {
        AnyValue::Vector(val.values.iter().map(|v| v.into()).collect())
    }
}
