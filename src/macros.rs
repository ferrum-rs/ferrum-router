/// Create and populate a router.
///
/// ```ignore
/// let router = router!(
///     get  "/"        => index        "index",
///     get  "/{query}" => queryHandler "query",
///     post "/"        => postHandler
/// );
/// ```
///
/// Is equivalent to:
///
/// ```ignore
/// let mut router = Router::new();
/// router.get("/", index, Id::some("index"));
/// router.get("/{query}", queryHandler, Id::some("query"));
/// router.post("/", postHandler, None);
/// ```
///
/// The method name must be lowercase, supported methods:
///
/// `get`, `post`, `put`, `delete`, `head`, `patch`, `options` and `any`.
#[macro_export]
macro_rules! router {
    ($($method:ident $glob:expr => $handler:tt $($route_id:expr)*),* $(,)*) => ({
        let mut router = $crate::Router::new();
        $(route_line!(router, $method $glob => $handler ($($route_id)*));)*
        router
    });
}

#[macro_export]
macro_rules! route_line {
    ($router:ident, $method:ident $glob:expr => $handler:tt () $(,)*) => {
        $router.$method($glob, $handler, None);
    };
    ($router:ident, $method:ident $glob:expr => $handler:tt ($route_id:expr) $(,)*) => {
        $router.$method($glob, $handler, $crate::Id::some($route_id));
    };
}

/// Generate a URI based off of the requested one.
///
/// ```ignore
/// uri_for!(request, "foo",
///          "query" => "test",
///          "extraparam" => "param")
/// ```
///
/// Is equivalent to:
///
/// ```ignore
/// ferrum_router::uri_for(&request, "foo", {
///     let mut params = ferrum_router:recognizer::Params::new();
///     params.insert("query".into(), "test".into());
///     params.insert("extraparam".into(), "param".into());
///     params
/// })
/// ```
#[macro_export]
macro_rules! uri_for {
    ($request:expr, $route_id:expr $(,$key:expr => $value:expr)* $(,)*) => (
        $crate::uri_for(&$request, $route_id, {
            // Underscore-prefix suppresses `unused_mut` warning
            // Also works on stable rust!
            let mut _params = $crate::Params::new();
            $(_params.insert($key.into(), $value.into());)*
            _params
        })
    )
}

#[cfg(test)]
mod tests {
    use ferrum::{Response, Request, FerrumResult, Method, Handler, Uri};
    use ferrum::request::HyperRequest;
    use recognizer::{DefaultStore, DefaultStoreBuild};

    //simple test to check that all methods expand without error
    #[test]
    fn test_methods_expand() {
        fn handler(_: &mut Request) -> FerrumResult<Response> { Ok(Response::new()) }
        let types = DefaultStore::with_default_types();

        let _ = router!(
            get "/" => handler "index",
            get "/{query}" => handler,
            post "/" => handler "post"
        );

        let _ = router!(
            get     "/foo" => handler,
            post    "/bar/" => handler "a",
            put     "/bar/baz" => handler "b",
            delete  "/bar/baz" => handler "c",
            head    "/foo" => handler "d",
            patch   "/bar/baz" => handler "e",
            options "/foo" => handler "f",
            any     "/" => handler "g",
            get     "/foo/{id:[0-9]+}" => handler "h",
            get     ("/foo/{name:string}", &types) => handler "i"
        );
    }

    #[test]
    fn test_uri_for() {
        fn handler(_: &mut Request) -> FerrumResult<Response> { Ok(Response::new()) }
        let router = router!(
            get "/foo" => handler "foo",
            get "/foo/{bar}" => handler "bar",
        );

        let mut request = Request::new(
            HyperRequest::new(Method::Get, "http://www.rust-lang.org/foo".parse().unwrap())
        );
        let _response = router.handle(&mut request);

        let uri: Uri = uri_for!(request, "foo",
                  "query" => "test",
                  "extraparam" => "param");
        assert_eq!("http://www.rust-lang.org/foo?extraparam=param&query=test", uri);

        let mut request = Request::new(
            HyperRequest::new(Method::Get, "http://www.rust-lang.org/foo/foo".parse().unwrap())
        );
        let _response = router.handle(&mut request);

        let uri: Uri = uri_for!(request, "bar",
                  "bar" => "test",
                  "query" => "param");
        assert_eq!("http://www.rust-lang.org/foo/test?query=param", uri);
    }
}
