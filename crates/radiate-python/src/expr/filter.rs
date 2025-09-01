#[derive(Debug, Clone)]
pub enum FilterExpr {
    Prob(f32),
}

impl FilterExpr {
    pub fn prob(p: f32) -> Self {
        FilterExpr::Prob(p)
    }
}
