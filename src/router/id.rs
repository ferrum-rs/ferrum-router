use std::borrow::Borrow;
use std::ops::Deref;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    #[inline]
    pub fn some<I>(id: I) -> Option<Id>
        where I: Into<Id>
    {
        Some(id.into())
    }

    #[inline]
    pub fn none() -> Option<Id> {
        None
    }
}

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<String> for Id {
    fn from(from: String) -> Self {
        Id(from)
    }
}

impl<'a> From<&'a str> for Id {
    fn from(from: &'a str) -> Self {
        Id(from.into())
    }
}

impl Into<String> for Id {
    fn into(self) -> String {
        self.0
    }
}

impl Deref for Id {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
        &self.0
    }
}
