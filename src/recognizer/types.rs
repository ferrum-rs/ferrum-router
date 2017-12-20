use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::cmp::Eq;
use std::hash::Hash;
use std::borrow::Borrow;
use std::marker::{Send, Sync};

pub type DefaultType = &'static str;

pub struct Type;

impl Type {
    pub const STRING: DefaultType = "[^/.]+";
    pub const NUMBER: DefaultType = "[0-9]+";
}

pub trait TypeName: Eq + Hash + Borrow<str> + Send + Sync {}
impl<T: Eq + Hash + Borrow<str> + Send + Sync> TypeName for T {}

pub trait TypePattern: AsRef<str> + Send + Sync {}
impl<T: AsRef<str> + Send + Sync> TypePattern for T {}

#[derive(Default, Debug)]
pub struct Types<N: TypeName, P: TypePattern>(pub HashMap<N, P>);

impl<N: TypeName, P: TypePattern> Deref for Types<N, P> {
    type Target = HashMap<N, P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N: TypeName, P: TypePattern> DerefMut for Types<N, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_types_value() {
        let mut types = Types::default();
        types.insert("key", "value");

        let value = types.get("key").unwrap();
        assert_eq!(*value, "value");
        let value: &str = types.get("key").unwrap().as_ref();
        assert_eq!(value, "value");

        let mut types = Types::default();
        types.insert("key", "value".to_string());

        let value = types.get("key").unwrap();
        assert_eq!(*value, "value".to_string());
        let value: &str = types.get("key").unwrap().as_ref();
        assert_eq!(value, "value");
    }
}