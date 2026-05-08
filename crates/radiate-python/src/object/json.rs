use pyo3::{
    Borrowed, Bound, PyAny, PyResult, Python,
    types::{PyAnyMethods, PyList},
};
use radiate_error::radiate_py_bail;

const NUMPY_ARRAY_CLASS: &str = "ndarray";

pub(super) fn try_to_json<'a, 'py>(
    python: Python<'py>,
    item: Borrowed<'a, 'py, PyAny>,
) -> PyResult<String> {
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

        return Ok(format!("[{}]", items.join(",")));
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
    let to_json = item.getattr("to_json").ok()?;
    let json_str = to_json.call0().ok()?;

    return json_str.extract::<String>().ok();
}

#[inline]
fn try_to_json_dumps(item: &Bound<'_, PyAny>) -> Option<String> {
    let json_module = item.py().import("json").ok()?;
    let dumps = json_module.getattr("dumps").ok()?;
    let json_str = dumps.call1((item,)).ok()?;

    return json_str.extract::<String>().ok();
}

#[inline]
fn try_to_json_numpy(item: &Bound<'_, PyAny>) -> Option<String> {
    let item_name = item
        .getattr("__class__")
        .ok()?
        .getattr("__name__")
        .ok()?
        .extract::<String>()
        .ok()?;

    if item_name == NUMPY_ARRAY_CLASS {
        let numpy_list = item.call_method0("tolist").ok()?;
        return try_to_json_fn(&numpy_list).or_else(|| try_to_json_dumps(&numpy_list));
    }

    None
}
