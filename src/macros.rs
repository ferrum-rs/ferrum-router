/// Create and populate a router.
///
/// ```ignore
/// let router = router!(index: get  "/"        => index,
///                      query: get  "/{query}" => queryHandler,
///                      post:  post "/"        => postHandler);
/// ```
///
/// Is equivalent to:
///
/// ```ignore
/// let mut router = Router::new();
/// router.get("/", index, "index");
/// router.get("/{query}", queryHandler, "query");
/// router.post("/", postHandler, "post");
/// ```
///
/// The method name must be lowercase, supported methods:
///
/// `get`, `post`, `put`, `delete`, `head`, `patch`, `options` and `any`.
#[macro_export]
macro_rules! router {
    ($($route_id:ident: $method:ident $glob:expr => $handler:expr),+ $(,)*) => ({
        let mut router = $crate::Router::new();
        $(router.$method($glob, $handler, stringify!($route_id));)*
        router
    });
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
///     let mut params = ::std::collections::BTreeMap::new();
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
            let mut _params = ::std::collections::BTreeMap::<String, String>::new();
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

        let _ = router!(a: get     "/foo" => handler,
                        b: post    "/bar/" => handler,
                        c: put     "/bar/baz" => handler,
                        d: delete  "/bar/baz" => handler,
                        e: head    "/foo" => handler,
                        f: patch   "/bar/baz" => handler,
                        g: options "/foo" => handler,
                        h: any     "/" => handler,
                        i: get     "/foo/{id:[0-9]+}" => handler,
                        j: get     ("/foo/{name:string}", &types) => handler);
    }

    #[test]
    fn test_uri_for() {
        fn handler(_: &mut Request) -> FerrumResult<Response> { Ok(Response::new()) }
        let router = router!(
            foo: get "/foo" => handler,
            bar: get "/foo/{bar}" => handler,
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
