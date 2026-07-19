use pyo3::{
    Borrowed, Bound, PyAny, PyResult, intern,
    types::{PyAnyMethods, PyList},
};
use radiate_error::radiate_py_bail;

const NUMPY_ARRAY_CLASS: &str = "ndarray";

pub(super) fn try_to_json<'a, 'py>(item: Borrowed<'a, 'py, PyAny>) -> PyResult<String> {
    if item.is_instance_of::<PyList>() {
        let mut items = Vec::new();
        for item in item.cast::<PyList>()?.try_iter()? {
            let item = item?;
            if let Some(json_str) = try_to_json_fn(&item) {
                items.push(json_str);
            } else if let Some(json_str) = try_to_json_dumps(&item) {
                items.push(json_str);
            } else if let Some(json_str) = try_to_json_numpy(&item) {
                items.push(json_str);
            } else {
                radiate_py_bail!(
                    "Failed to serialize PyAnyObject: item does not have to_json method"
                );
            }
        }

        radiate_py_bail!("Failed to serialize PyAnyObject: list items do not have to_json method");
    } else if let Some(json_str) = try_to_json_fn(&item) {
        return Ok(json_str);
    } else if let Some(json_str) = try_to_json_dumps(&item) {
        return Ok(json_str);
    } else if let Some(json_str) = try_to_json_numpy(&item) {
        return Ok(json_str);
    }

    radiate_py_bail!("Failed to serialize PyAnyObject: unsupported type");
}

#[inline]
fn try_to_json_fn(item: &Bound<'_, PyAny>) -> Option<String> {
    let py = item.py();
    let to_json = item.getattr(intern!(py, "to_json")).ok()?;
    let json_str = to_json.call0().ok()?;

    json_str.extract::<String>().ok()
}

#[inline]
fn try_to_json_dumps(item: &Bound<'_, PyAny>) -> Option<String> {
    let py = item.py();
    let json_module = py.import(intern!(py, "json")).ok()?;
    let dumps = json_module.getattr(intern!(py, "dumps")).ok()?;
    let json_str = dumps.call1((item,)).ok()?;

    json_str.extract::<String>().ok()
}

#[inline]
fn try_to_json_numpy(item: &Bound<'_, PyAny>) -> Option<String> {
    let py = item.py();
    let item_name = item
        .getattr(intern!(py, "__class__"))
        .ok()?
        .getattr(intern!(py, "__name__"))
        .ok()?
        .extract::<String>()
        .ok()?;

    if item_name == NUMPY_ARRAY_CLASS {
        let numpy_list = item.call_method0(intern!(py, "tolist")).ok()?;
        return try_to_json_fn(&numpy_list).or_else(|| try_to_json_dumps(&numpy_list));
    }

    None
}
