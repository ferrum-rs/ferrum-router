Ferrum Router
=============

> Routing handler for the [Ferrum](https://github.com/ferrum-rs/ferrun) web framework.

Ferrum Router is a fast, convenient, and flexible routing middleware for Ferrum. It
allows complex glob regex patterns and named url parameters and also allows handlers
to be any Handler, including all Chains.

## Example

```rust
extern crate ferrum;
extern crate ferrum_router;

use ferrum::*;
use ferrum_router::{Router, Id};

fn main() {
    let mut router = Router::new();                     // Alternative syntax:
                                                        // let router = router!(
    router.get("/", handler, None);                     //     get "/" => handler,
    router.get("/{query}", handler, Id::some("query")); //     get "/{query}" => handler "query");

    Ferrum::new(router).http("localhost:3000").unwrap();

    fn handler(request: &mut Request) -> FerrumResult<Response> {
        let params = request.extensions.get::<Router>().unwrap();
        let query = params.get("query").map(|value| value.as_str()).unwrap_or("/");
        
        Ok(Response::new().with_content(query, mime::TEXT_PLAIN))
    }
}
```

## Overview

Router is a part of Ferrum's [core bundle](https://github.com/ferrum-rs/core).

- Route client requests based on their paths
- Parse parameters and provide them to other middleware/handlers

## Installation

If you're using cargo, just add ferrum-router to your `Cargo.toml`.

```toml
[dependencies]

ferrum-router = "*"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Examples](/examples)

Check out the [examples](/examples) directory!

You can run an individual example using `cargo run --example example-name`.
Note that for benchmarking you should make sure to use the `--release` flag,
which will cause cargo to compile the entire toolchain with optimizations.
Without `--release` you will get truly sad numbers.

## Benches

To run benchmark tests, please use Rust nightly toolchain:

```
rustup default nightly
cargo bench --features "nightly"
```

## License

MIT
