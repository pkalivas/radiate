use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python, exceptions::PyValueError,
};
use radiate::object::AnyValue;

pub(crate) fn reinterpret_vec<T: Transparent>(input: Vec<T>) -> Vec<T::Target> {
    assert_eq!(size_of::<T>(), size_of::<T::Target>());
    assert_eq!(align_of::<T>(), align_of::<T::Target>());
    let len = input.len();
    let cap = input.capacity();
    let mut manual_drop_vec = std::mem::ManuallyDrop::new(input);
    let vec_ptr: *mut T = manual_drop_vec.as_mut_ptr();
    let ptr: *mut T::Target = vec_ptr as *mut T::Target;
    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}

pub(crate) fn vec_extract_wrapped<T>(buf: Vec<Wrap<T>>) -> Vec<T> {
    reinterpret_vec(buf)
}

/// # Safety
/// Should only be implemented for transparent types
pub(crate) unsafe trait Transparent {
    type Target;
}

unsafe impl<T> Transparent for Wrap<T> {
    type Target = T;
}

unsafe impl<T: Transparent> Transparent for Option<T> {
    type Target = Option<T::Target>;
}

#[repr(transparent)]
pub struct Wrap<T>(pub T);

impl<T> Clone for Wrap<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Wrap(self.0.clone())
    }
}
impl<T> From<T> for Wrap<T> {
    fn from(t: T) -> Self {
        Wrap(t)
    }
}

impl<'py> FromPyObject<'py> for Wrap<AnyValue<'py>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        super::py_object_to_any_value(ob, true).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
        .map(Wrap)
    }
}

impl<'py> IntoPyObject<'py> for Wrap<AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        super::any_value_into_py_object(self.0, py)
    }
}

impl<'py> IntoPyObject<'py> for &Wrap<AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.clone().into_pyobject(py)
    }
}
