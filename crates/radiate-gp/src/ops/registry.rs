// crates/radiate-gp/src/ops/registry.rs
use crate::{Arity, Op};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

type OpFactory<T> = Arc<dyn Fn() -> Op<T> + Send + Sync>;

struct OpRegistry<T> {
    ops: HashMap<&'static str, OpFactory<T>>,
}

impl<T> Default for OpRegistry<T> {
    fn default() -> Self {
        Self {
            ops: HashMap::new(),
        }
    }
}

type Boxed = Box<dyn Any + Send + Sync>;
static REGISTRIES: OnceLock<RwLock<HashMap<TypeId, Boxed>>> = OnceLock::new();

fn registries() -> &'static RwLock<HashMap<TypeId, Boxed>> {
    REGISTRIES.get_or_init(|| RwLock::new(HashMap::new()))
}

fn with_registry_mut<T, R>(f: impl FnOnce(&mut OpRegistry<T>) -> R) -> R
where
    T: 'static,
{
    let map = registries();
    let mut w = map.write().unwrap();
    let entry = w
        .entry(TypeId::of::<T>())
        .or_insert_with(|| Box::new(OpRegistry::<T>::default()));
    let reg = entry
        .downcast_mut::<OpRegistry<T>>()
        .expect("type map downcast");
    f(reg)
}

fn with_registry<T: 'static, R>(f: impl FnOnce(&OpRegistry<T>) -> R) -> R {
    let map = registries();
    let r = map.read().unwrap();
    let Some(entry) = r.get(&TypeId::of::<T>()) else {
        return f(&OpRegistry::<T>::default());
    };
    let reg = entry
        .downcast_ref::<OpRegistry<T>>()
        .expect("type map downcast");
    f(reg)
}

// Public API
pub fn register_op<T>(name: &'static str, factory: impl Fn() -> Op<T> + Send + Sync + 'static)
where
    T: 'static,
{
    with_registry_mut::<T, _>(|reg| {
        reg.ops.insert(name, Arc::new(factory));
    });
}

pub fn get_op<T>(name: &str) -> Option<Op<T>>
where
    T: 'static,
{
    with_registry::<T, _>(|reg| reg.ops.get(name).map(|f| f()))
}

// Ergonomic helpers
pub fn register_const<T>(name: &'static str, value: T)
where
    T: Clone + Send + Sync + 'static,
{
    register_op::<T>(name, move || Op::Const(name, value.clone()));
}

pub fn register_fn<T>(name: &'static str, arity: Arity, body: fn(&[T]) -> T)
where
    T: 'static,
{
    register_op::<T>(name, move || Op::Fn(name, arity, body));
}

pub fn register_fn_any<T>(name: &'static str, body: fn(&[T]) -> T)
where
    T: 'static,
{
    register_op::<T>(name, move || Op::Fn(name, Arity::Any, body));
}

pub fn register_op_value<T>(op: Op<T>)
where
    T: Clone + Send + Sync + 'static,
{
    let name = radiate_core::intern!(op.name());
    // only register if absent
    if crate::ops::registry::get_op::<T>(name).is_none() {
        register_op::<T>(name, move || op.clone());
    }
}

// Optional macros with explicit type parameter
#[macro_export]
macro_rules! op_get {
    (<$t:ty> $name:literal) => {
        $crate::ops::registry::get_op::<$t>($name)
            .unwrap_or_else(|| panic!("unknown op `{}` for type {}", $name, stringify!($t)))
    };
}

#[macro_export]
macro_rules! op_register_const {
    (<$t:ty> $name:literal, $value:expr) => {
        $crate::ops::registry::register_const::<$t>($name, $value);
    };
}

#[macro_export]
macro_rules! op_register_fn {
    (<$t:ty> $name:literal, $arity:expr, $body:expr) => {
        $crate::ops::registry::register_fn::<$t>($name, $arity, $body);
    };
}

#[macro_export]
macro_rules! op_register_fn_any {
    (<$t:ty> $name:literal, $body:expr) => {
        $crate::ops::registry::register_fn_any::<$t>($name, $body);
    };
}

#[macro_export]
macro_rules! op {
    // 1) Get by name (typed), return Op<T>
    (<$t:ty> $name:literal) => {{
        $crate::ops::registry::get_op::<$t>($name)
            .unwrap_or_else(|| panic!("unknown op `{}` for type {}", $name, stringify!($t)))
    }};

    // 2) Register typed function with explicit arity, return Op<T>
    ($name:literal, $arity:expr, $body:expr) => {{
        $crate::ops::registry::register_fn::<_>($name, $arity, $body);
        $crate::ops::registry::get_op::<_>($name).unwrap()
    }};

    // 3) Register typed constant, return Op<T>
    (<$t:ty> $name:literal, $value:expr) => {{
        $crate::ops::registry::register_const::<$t>($name, $value);
        $crate::ops::registry::get_op::<$t>($name).unwrap()
    }};

    // 4) Optional: variadic function helper
    (fn_any <$t:ty> $name:literal, $body:expr) => {{
        $crate::ops::registry::register_fn_any::<$t>($name, $body);
        $crate::ops::registry::get_op::<$t>($name).unwrap()
    }};

    // 5) Register from an Op<T> value, return it (keep this last)
    ($op:expr) => {{
        let __op = $op;
        $crate::ops::registry::register_op_value(__op.clone());
        __op
    }};
}
