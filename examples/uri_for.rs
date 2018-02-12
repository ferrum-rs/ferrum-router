extern crate ferrum;
#[macro_use]
extern crate ferrum_router;

// To run, $ cargo run --example uri_for
// Go to http://localhost:3000 to see "Please go to: /test?extraparam=foo", dynamically generated
// from the original route.
// Go to http://localhost:3000/test to see "test".
// Go to http://localhost:3000/foo to see "foo".

use ferrum::{Ferrum, FerrumResult, mime, Request, Response};
use ferrum_router::{Router};

fn main() {
    let router = router! {
        get "/" => handler,
        get "/{query}" => query_handler "query_route"
    };

    Ferrum::new(router).http("localhost:3000").unwrap();

    fn handler(request: &mut Request) -> FerrumResult<Response> {
        Ok(Response::new().with_content(
            format!("Please go to: {}",
                    uri_for!(
                        request, "query_route",
                        "query" => "test",
                        "extraparam" => "foo"
                    )
            ),
            mime::TEXT_PLAIN
        ))
    }

    fn query_handler(request: &mut Request) -> FerrumResult<Response> {
        let query = request.extensions.get::<Router>().unwrap()
            .get("query").map(AsRef::<str>::as_ref).unwrap_or("/");
        Ok(Response::new().with_content(query, mime::TEXT_PLAIN))
    }
}
