use std::collections::HashMap;
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

pub type Store<N, P> = HashMap<N, P>;

pub type DefaultStore = Store<NameDefaultType, PatternDefaultType>;

pub trait GlobTypes {
    type Name: TypeName;
    type Pattern: TypePattern;

    fn store(&self) -> &Store<Self::Name, Self::Pattern>;
}

pub trait GlobTypesMut: GlobTypes {
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
}

impl<N, P> GlobTypesMut for Store<N, P>
    where N: TypeName,
          P: TypePattern
{
    fn store_mut(&mut self) -> &mut Store<Self::Name, Self::Pattern> {
        self
    }
}

impl<'a, N, P> GlobTypes for &'a Store<N, P>
    where N: TypeName,
          P: TypePattern
{
    type Name = N;
    type Pattern = P;

    fn store(&self) -> &Store<Self::Name, Self::Pattern> {
        *self
    }
}

impl<'a, N, P> GlobTypes for &'a mut Store<N, P>
    where N: TypeName,
          P: TypePattern
{
    type Name = N;
    type Pattern = P;

    fn store(&self) -> &Store<Self::Name, Self::Pattern> {
        *self
    }
}

impl<'a, N, P> GlobTypesMut for &'a mut Store<N, P>
    where N: TypeName,
          P: TypePattern
{
    fn store_mut(&mut self) -> &mut Store<Self::Name, Self::Pattern> {
        &mut **self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_glob_types_value<'a, T>(types: &'a T, key: <T as GlobTypes>::Name) -> &'a <T as GlobTypes>::Pattern
        where T: GlobTypes
    {
        types.store().get(key.borrow()).unwrap()
    }

    fn equal_glob_types_value<T>(types: T, key: <T as GlobTypes>::Name, expected: <T as GlobTypes>::Pattern)
        where T: GlobTypes,
    {
        let value = types.store().get(key.borrow()).unwrap();
        assert_eq!(value.as_ref(), expected.as_ref());
    }

    #[test]
    fn use_different_glob_types() {
        let mut types = Store::default();
        types.insert("key", "value");

        let value = types.get("key").unwrap();
        assert_eq!(*value, "value");

        let value = get_glob_types_value(&types, "key");
        assert_eq!(*value, "value");

        equal_glob_types_value(&types, "key", "value");
        equal_glob_types_value(types.clone(), "key", "value");


        let mut types = Store::default();
        types.insert("key", "value".to_string());

        let value = types.get("key").unwrap();
        assert_eq!(*value, "value".to_string());

        let value = get_glob_types_value(&types, "key");
        assert_eq!(*value, "value".to_string());

        equal_glob_types_value(&types, "key", "value".to_string());
        equal_glob_types_value(types.clone(), "key", "value".to_string());
    }
}