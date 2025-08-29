use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Field {
    pub name: String,
}

impl Field {
    pub fn new(name: String) -> Self {
        Field { name }
    }

    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl From<Field> for (String, Field) {
    fn from(value: Field) -> Self {
        (value.name.clone(), value)
    }
}

impl From<String> for Field {
    fn from(name: String) -> Self {
        Field::new(name)
    }
}

impl From<&str> for Field {
    fn from(name: &str) -> Self {
        Field::new(name.to_string())
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field {{\n name: {},\n }}", self.name)
    }
}
