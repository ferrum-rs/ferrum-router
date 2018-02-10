extern crate ferrum;
#[macro_use]
extern crate ferrum_router;

// To run, $ cargo run --example simple_with_macro
// To use, go to http://localhost:3000/test and see output "test"
// Or, go to http://localhost:3000 to see a default "OK"

use ferrum::{Ferrum, FerrumResult, mime, Request, Response};
use ferrum_router::{Router};

fn main() {
    let router = router!(root: get "/" => handler, query: get "/{query}" => query_handler);

    Ferrum::new(router).http("localhost:3000").unwrap();

    fn handler(_: &mut Request) -> FerrumResult<Response> {
        Ok(Response::new().with_content("OK", mime::TEXT_PLAIN))
    }

    fn query_handler(request: &mut Request) -> FerrumResult<Response> {
        let query = request.extensions.get::<Router>().unwrap()
            .get("query").map(|value| value.as_str()).unwrap_or("/");
        Ok(Response::new().with_content(query, mime::TEXT_PLAIN))
    }
}
