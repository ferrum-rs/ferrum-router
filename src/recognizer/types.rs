use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub type TypesCollection<'a> = HashMap<&'a str, &'a str>;

#[derive(Default)]
pub struct Types<'a>(pub TypesCollection<'a>);

impl<'a> Types<'a> {
    pub const STRING_TYPE: &'static str = "[^/.]+";
    pub const NUMBER_TYPE: &'static str = "[0-9]+";

    pub fn default_type() -> &'static str {
        Types::STRING_TYPE
    }
}

impl<'a> Deref for Types<'a> {
    type Target = TypesCollection<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Types<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
