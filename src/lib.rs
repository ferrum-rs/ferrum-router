//#![deny(missing_docs)]
//#![cfg_attr(test, deny(warnings))]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

//! `Router` provides fast and flexible routing for Ferrum.

extern crate ferrum;
//extern crate route_recognizer as recognizer;
extern crate url;
extern crate regex;

//pub use router::{Router, NoRoute, TrailingSlash};
//pub use recognizer::Params;
//pub use url_for::url_for;

mod router;
mod recognizer;
//mod macros;
//mod url_for;
