extern crate ferrum;
extern crate ferrum_router;

// To run, $ cargo run --example simple_with_types
// To use, go to http://localhost:3000/test and see output "test"
// Or, go to http://localhost:3000/123 to see a "id: 123"

use std::str::FromStr;

use ferrum::{Ferrum, FerrumResult, mime, Request, Response};
use ferrum_router::{Router};
use ferrum_router::recognizer::{DefaultStore, DefaultStoreBuild};

fn handler(_: &mut Request) -> FerrumResult<Response> {
    Ok(Response::new().with_content("OK", mime::TEXT_PLAIN))
}

fn id_handler(request: &mut Request) -> FerrumResult<Response> {
    let params = request.extensions.get::<Router>().unwrap();
    let id = i32::from_str(
        params.get("id")
            .map(String::as_str)
            .unwrap_or("0")
    ).unwrap_or(0);

    Ok(Response::new().with_content(format!("id: {}", id), mime::TEXT_PLAIN))
}

fn query_handler(request: &mut Request) -> FerrumResult<Response> {
    let params = request.extensions.get::<Router>().unwrap();
    let query = params.get("query")
        .map(String::as_str)
        .unwrap_or("/");

    Ok(Response::new().with_content(query, mime::TEXT_PLAIN))
}

fn main() {
    let mut types = DefaultStore::with_default_types();
    types.insert("id_type", "[1-9][0-9]*");

    let mut router = Router::new();
    router.get("/", handler, None);
    router.get(("/{id:id_type}", &types), id_handler, None);
    router.get(("/{query:string}", &types), query_handler, None);

    Ferrum::new(router).http("localhost:3000").unwrap();
}
