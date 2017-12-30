extern crate ferrum;
extern crate ferrum_router;

use ferrum::{Handler, Ferrum, FerrumResult, mime, Request, Response};
use ferrum_router::Router;

struct MessageHandler {
    message: String
}

impl Handler for MessageHandler {
    fn handle(&self, _: &mut Request) -> FerrumResult<Response> {
        Ok(Response::new().with_content(self.message.clone(), mime::TEXT_PLAIN))
    }
}

fn main() {
    let handler = MessageHandler {
        message: "You've found the index page!".to_string()
    };

    let mut router = Router::new();
    router.get("/", handler, "index");

    Ferrum::new(router).http("localhost:3000").unwrap();
}
