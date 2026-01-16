use crate::Factory;
use std::fmt::Debug;

#[derive(Hash, Clone)]
pub struct Param<T> {
    data: T,
    supplier: fn(&T) -> T,
    modifier: fn(&mut T),
}

impl<T> Param<T> {
    pub fn new(data: T, supplier: fn(&T) -> T, modifier: fn(&mut T)) -> Self {
        Param {
            data,
            supplier,
            modifier,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn supplier(&self) -> fn(&T) -> T {
        self.supplier
    }

    pub fn modifier(&self) -> fn(&mut T) {
        self.modifier
    }
}

impl<T> Factory<(), Param<T>> for Param<T> {
    fn new_instance(&self, _: ()) -> Param<T> {
        let data = (self.supplier)(&self.data);
        Param {
            data,
            supplier: self.supplier,
            modifier: self.modifier,
        }
    }
}

impl<T> Default for Param<T>
where
    T: Default,
{
    fn default() -> Self {
        Param {
            data: T::default(),
            supplier: |_| T::default(),
            modifier: |_| {},
        }
    }
}

impl<T> PartialEq for Param<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Debug for Param<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

// impl<T> Factory<T, Param<T>> for Param<T> {
//     fn new_instance(&self, mut val: T) -> Param<T> {
//         (self.modifier)(&mut val);
//         Param {
//             data: val,
//             supplier: self.supplier,
//             modifier: self.modifier,
//         }
//     }
// }
