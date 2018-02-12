extern crate ferrum;
extern crate ferrum_router;

// To run, $ cargo run --example simple
// To use, go to http://localhost:3000/test and see output "test"
// Or, go to http://localhost:3000 to see a default "OK"

use ferrum::{Ferrum, FerrumResult, mime, Request, Response};
use ferrum_router::{Router};

fn handler(_: &mut Request) -> FerrumResult<Response> {
    Ok(Response::new().with_content("OK", mime::TEXT_PLAIN))
}

fn query_handler(request: &mut Request) -> FerrumResult<Response> {
    let params = request.extensions.get::<Router>().unwrap();
    let query = params.get("query").map(|value| value.as_str()).unwrap_or("/");
    Ok(Response::new().with_content(query, mime::TEXT_PLAIN))
}

fn main() {
    let mut router = Router::new();
    router.get("/", handler, None);
    router.get("/{query}", query_handler, None);

    Ferrum::new(router).http("localhost:3000").unwrap();
}
