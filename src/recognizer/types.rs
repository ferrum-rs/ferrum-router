use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::cmp::Eq;
use std::hash::Hash;
use std::borrow::Borrow;
use std::marker::{Send, Sync};

pub type NameDefaultType = &'static str;
pub type PatternDefaultType = &'static str;

pub struct Type;

impl Type {
    pub const STRING_NAME: NameDefaultType = "string";
    pub const STRING_PATTERN: PatternDefaultType = "[^/.]+";

    pub const NUMBER_NAME: NameDefaultType = "number";
    pub const NUMBER_PATTERN: PatternDefaultType = "[0-9]+";
}

pub trait TypeName: Eq + Hash + Borrow<str> + Send + Sync {}
impl<T: Eq + Hash + Borrow<str> + Send + Sync> TypeName for T {}

pub trait TypePattern: AsRef<str> + Send + Sync {}
impl<T: AsRef<str> + Send + Sync> TypePattern for T {}


pub type DefaultTypes = Types<NameDefaultType, PatternDefaultType>;

#[derive(Default, Debug)]
pub struct Types<N, P>(pub HashMap<N, P>)
    where N: TypeName,
          P: TypePattern;

impl<N, P> Deref for Types<N, P>
    where N: TypeName,
          P: TypePattern
{
    type Target = HashMap<N, P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N, P> DerefMut for Types<N, P>
    where N: TypeName,
          P: TypePattern
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type Store<N, P> = HashMap<N, P>;

pub trait GlobTypes {
    type Name: TypeName;
    type Pattern: TypePattern;

    fn store(&self) -> &Store<Self::Name, Self::Pattern>;

    fn store_mut(&mut self) -> &mut Store<Self::Name, Self::Pattern>;
}

impl<N, P> GlobTypes for Store<N, P>
    where N: TypeName,
          P: TypePattern
{
    type Name = N;
    type Pattern = P;

    fn store(&self) -> &Store<Self::Name, Self::Pattern> {
        self
    }

    fn store_mut(&mut self) -> &mut Store<Self::Name, Self::Pattern> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glob_types_value<'a, T>(types: &'a T, key: <T as GlobTypes>::Name) -> &'a <T as GlobTypes>::Pattern
        where T: GlobTypes
    {
        types.store().get(key.borrow()).unwrap()
    }

    #[test]
    fn glob_types() {
        let mut types = Store::default();
        types.insert("key", "value");

        let value = types.get("key").unwrap();
        assert_eq!(*value, "value");

        let value = glob_types_value(&types, "key");
        assert_eq!(*value, "value");
    }

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