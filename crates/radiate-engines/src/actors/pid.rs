use radiate_core::SmallStr;

const SEPARATOR: &str = "/";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessId(SmallStr);

impl ProcessId {
    pub const fn new_const(id: SmallStr) -> Self {
        ProcessId(id)
    }

    pub fn new(id: impl AsRef<str>) -> Self {
        ProcessId(SmallStr::from(id.as_ref()))
    }

    pub fn child(&self, child_id: impl AsRef<str>) -> Self {
        let joined = format!("{}{}{}", self.0, SEPARATOR, child_id.as_ref());
        ProcessId(SmallStr::from(joined))
    }
}

impl From<&str> for ProcessId {
    fn from(s: &str) -> Self {
        ProcessId::new(s)
    }
}

impl From<String> for ProcessId {
    fn from(s: String) -> Self {
        ProcessId::new(s)
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
