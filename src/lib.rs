//#![deny(missing_docs)]
//#![cfg_attr(test, deny(warnings))]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

//! This provides fast and flexible routing for Ferrum.

extern crate ferrum;
extern crate url;
extern crate regex;

pub use router::{Router, NoRoute, Id};
pub use recognizer::{Recognize, Recognizer, Params};
pub use uri_for::{UriFor, uri_for};

pub mod router;
pub mod recognizer;
pub mod macros;
pub mod uri_for;
