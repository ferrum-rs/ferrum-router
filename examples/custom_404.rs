extern crate ferrum;
extern crate ferrum_router;

// To run, $ cargo run --example custom_404
// To use, go to http://localhost:3000/foobar to see the custom 404
// Or, go to http://localhost:3000 for a standard 200 OK

use ferrum::{AfterMiddleware, Chain, Ferrum, FerrumError, FerrumResult, mime, Request, Response, StatusCode};
use ferrum_router::{Router, NoRoute};

struct Custom404;

impl AfterMiddleware for Custom404 {
    fn catch(&self, _: &mut Request, error: FerrumError) -> FerrumResult<Response> {
        println!("Hitting custom 404 middleware");

        if error.error.is::<NoRoute>() {
            Ok(Response::new()
                .with_content("Custom 404 response", mime::TEXT_PLAIN)
                .with_status(StatusCode::NotFound))
        } else {
            Err(error)
        }
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/", handler, None);

    let mut chain = Chain::new(router);
    chain.link_after(Custom404);

    Ferrum::new(chain).http("localhost:3000").unwrap();
}

fn handler(_: &mut Request) -> FerrumResult<Response> {
    Ok(Response::new().with_content("Handling response", mime::TEXT_PLAIN))
}
