use super::{SelectExpr, select::PathSegment};
use radiate_utils::{AnyValue, Field};
use std::collections::HashMap;

pub trait ExprProjection {
    fn project(&self, path: &SelectExpr) -> Option<AnyValue<'static>>;
}

impl<T> ExprProjection for Vec<T>
where
    T: Clone + Into<AnyValue<'static>>,
{
    fn project(&self, selector: &SelectExpr) -> Option<AnyValue<'static>> {
        match selector {
            SelectExpr::Path(path) => {
                let mut current = AnyValue::Vector(self.iter().cloned().map(Into::into).collect());

                for segment in path {
                    current = match segment {
                        PathSegment::Index(i) => current.get_index(*i)?,
                        _ => return None,
                    };
                }

                Some(current)
            }
            SelectExpr::Nth(n) => self.get(*n).cloned().map(Into::into),
            SelectExpr::Element => Some(AnyValue::Vector(
                self.iter().cloned().map(Into::into).collect(),
            )),
            _ => None,
        }
    }
}

impl<T> ExprProjection for HashMap<String, T>
where
    T: Clone + Into<AnyValue<'static>>,
{
    fn project(&self, selector: &SelectExpr) -> Option<AnyValue<'static>> {
        match selector {
            SelectExpr::Path(path) => {
                let mut current = AnyValue::Struct(
                    self.iter()
                        .map(|(k, v)| {
                            let cloned_value = v.clone().into();
                            (Field::new(k.into(), cloned_value.dtype()), cloned_value)
                        })
                        .collect(),
                );

                for segment in path {
                    current = match segment {
                        PathSegment::Key(key) => current.get_key(key)?,
                        PathSegment::Index(i) => current.get_index(*i)?,
                        PathSegment::StructField(field) => current.get_field(field)?,
                    };
                }

                Some(current)
            }
            SelectExpr::Field(key, field) => {
                let value = self.get(&key.clone().into_string()?)?.clone().into();
                value.get_field(field)
            }
            _ => None,
        }
    }
}

impl<'a> ExprProjection for AnyValue<'a> {
    fn project(&self, selector: &SelectExpr) -> Option<AnyValue<'static>> {
        match selector {
            SelectExpr::Path(path) => {
                let mut current = self.clone();

                for segment in path {
                    current = match segment {
                        PathSegment::Key(key) => current.get_key(key)?,
                        PathSegment::Index(i) => current.get_index(*i)?,
                        PathSegment::StructField(field) => current.get_field(field)?,
                    };
                }

                Some(current.into_static())
            }
            SelectExpr::Field(key, field) => {
                let value = self.get_key(key)?.into_static();
                value.get_field(field)
            }
            SelectExpr::Nth(n) => self.get_index(*n).map(|v| v.into_static()),
            SelectExpr::Element => Some(self.clone().into_static()),
        }
    }
}

impl ExprProjection for f32 {
    fn project(&self, _: &SelectExpr) -> Option<AnyValue<'static>> {
        Some(AnyValue::Float32(*self))
    }
}

impl ExprProjection for i32 {
    fn project(&self, _: &SelectExpr) -> Option<AnyValue<'static>> {
        Some(AnyValue::Int32(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Evaluate, Expr, expr, select::PathBuilder};
    use radiate_utils::{AnyValue, Field};
    use std::collections::HashMap;

    fn f32_of(value: AnyValue<'_>) -> f32 {
        value.extract::<f32>().unwrap()
    }

    fn i32_of(value: AnyValue<'_>) -> i32 {
        value.extract::<i32>().unwrap()
    }

    #[test]
    fn vec_projection_supports_nth() {
        let values = vec![10i32, 20, 30];
        let mut selector = SelectExpr::Nth(1);

        let result = selector.eval(&values).unwrap();

        assert_eq!(i32_of(result), 20);
    }

    #[test]
    fn vec_projection_supports_path_index() {
        let values = vec![10i32, 20, 30];
        let mut selector = SelectExpr::Path(vec![PathSegment::Index(2)]);

        let result = selector.eval(&values).unwrap();

        assert_eq!(i32_of(result), 30);
    }

    #[test]
    fn vec_projection_invalid_path_returns_null() {
        let values = vec![10i32, 20, 30];
        let mut selector =
            SelectExpr::Path(vec![PathSegment::Key(AnyValue::from("nope").into_static())]);

        let result = selector.eval(&values).unwrap_or(AnyValue::Null);

        assert!(matches!(result, AnyValue::Null));
    }

    #[test]
    fn hashmap_projection_supports_field() {
        let mut inner = HashMap::new();
        inner.insert("mean".to_string(), 12.5f32);

        let mut selector: Expr = expr::path("mean").into();

        let result = selector.eval(&inner).unwrap();

        assert_eq!(f32_of(result), 12.5);
    }

    #[test]
    fn hashmap_projection_supports_path_key() {
        let mut map = HashMap::new();
        map.insert("accuracy".to_string(), 0.91f32);

        let mut selector = SelectExpr::Path(vec![PathSegment::Key(
            AnyValue::from("accuracy").into_static(),
        )]);

        let result = selector.eval(&map).unwrap();

        assert_eq!(f32_of(result), 0.91);
    }

    #[test]
    fn hashmap_invalid_key_returns_null() {
        let mut map = HashMap::new();
        map.insert("accuracy".to_string(), 0.91f32);

        let mut selector = SelectExpr::Path(vec![PathSegment::Key(
            AnyValue::from("missing").into_static(),
        )]);

        let result = selector.eval(&map).unwrap_or(AnyValue::Null);

        assert!(matches!(result, AnyValue::Null));
    }

    #[test]
    fn nested_hashmap_vec_hashmap_path_works() {
        let mut user1 = HashMap::new();
        user1.insert("name".to_string(), AnyValue::from("alice").into_static());
        user1.insert("score".to_string(), AnyValue::from(10.0f32).into_static());

        let mut user2 = HashMap::new();
        user2.insert("name".to_string(), AnyValue::from("bob").into_static());
        user2.insert("score".to_string(), AnyValue::from(25.0f32).into_static());

        let users = vec![
            AnyValue::Struct(
                user1
                    .iter()
                    .map(|(k, v)| (Field::new(k.clone().into(), v.dtype()), v.clone()))
                    .collect(),
            ),
            AnyValue::Struct(
                user2
                    .iter()
                    .map(|(k, v)| (Field::new(k.clone().into(), v.dtype()), v.clone()))
                    .collect(),
            ),
        ];

        let mut root = HashMap::new();
        root.insert("users".to_string(), AnyValue::Vector(users));

        let mut selector = SelectExpr::Path(vec![
            PathSegment::Key(AnyValue::from("users").into_static()),
            PathSegment::Index(1),
            PathSegment::Key(AnyValue::from("name").into_static()),
        ]);

        let result = selector.eval(&root).unwrap();

        match result {
            AnyValue::Str(s) => assert_eq!(s, "bob"),
            AnyValue::StrOwned(s) => assert_eq!(s, "bob"),
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn path_builder_builds_selector_expr() {
        let expr: Expr = PathBuilder::default()
            .key("users")
            .index(0)
            .key("name")
            .into();

        match expr {
            Expr::Selector(SelectExpr::Path(path)) => {
                assert_eq!(path.len(), 3);
                assert!(matches!(&path[0], PathSegment::Key(_)));
                assert!(matches!(&path[1], PathSegment::Index(0)));
                assert!(matches!(&path[2], PathSegment::Key(_)));
            }
            other => panic!("expected Expr::Selector(Path), got {other:?}"),
        }
    }

    #[test]
    fn nested_numeric_path_can_be_compared_through_expr_tree() {
        let mut inner = HashMap::new();
        inner.insert("value".to_string(), 7.0f32);

        let mut root = HashMap::new();
        root.insert("metric".to_string(), inner);

        let mut expr: Expr = PathBuilder::default().key("metric").key("value").into();
        expr = expr.gt(5.0);

        let result = expr.eval(&root).unwrap();

        match result {
            AnyValue::Bool(v) => assert!(v),
            other => panic!("expected bool result, got {other:?}"),
        }
    }
}
