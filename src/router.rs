use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use ferrum::{Request, Response, Handler, FerrumResult, FerrumError};
use ferrum::{header, Method, StatusCode};
use ferrum::typemap::Key;

use recognizer::{Glob, GlobTypes, Recognizer, Recognize, RouteMatch, Params};

pub struct RouterInner {
    /// The routers, specialized by method.
    pub routers: HashMap<Method, Vec<Recognizer>>,

    /// Routes that accept any method.
    pub wildcard: Vec<Recognizer>,

    /// Used in URL generation.
    pub route_ids: HashMap<String, String>,
}

/// `Router` provides an interface for creating complex routes as middleware
/// for the Ferrum framework.
pub struct Router {
    inner: Arc<RouterInner>
}

impl Router {
    /// Construct a new, empty `Router`.
    ///
    /// ```ignore
    /// use ferrum_router::Router;
    /// let router = Router::new();
    /// ```
    pub fn new() -> Router {
        Router {
            inner: Arc::new(RouterInner {
                routers: HashMap::new(),
                wildcard: Vec::new(),
                route_ids: HashMap::new(),
            })
        }
    }

    fn mut_inner(&mut self) -> &mut RouterInner {
        Arc::get_mut(&mut self.inner).expect("Cannot modify router at this point.")
    }

    /// Add a new route to a `Router`, matching both a method and glob pattern.
    ///
    /// `route` supports glob patterns: `*` for a single wildcard segment and
    /// `:param` for matching storing that segment of the request url in the `Params`
    /// object, which is stored in the request `extensions`.
    ///
    /// For instance, to route `Get` requests on any route matching
    /// `/users/:userid/:friend` and store `userid` and `friend` in
    /// the exposed Params object:
    ///
    /// ```ignore
    /// let mut router = Router::new();
    /// router.route(Method::Get, "/users/:userid/:friendid", controller, "user_friend");
    /// ```
    ///
    /// `route_id` is a unique name for your route, and is used when generating an URL with
    /// `url_for`.
    ///
    /// The controller provided to route can be any `Handler`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Handler`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<G, H, I, S, T>(&mut self, method: Method, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        let glob = glob.into();
        let types = glob.types().map(|types| types.store());

        self.mut_inner().routers
            .entry(method)
            .or_insert(Vec::new())
            .push(Recognizer::new(glob.path(), Box::new(handler), types).unwrap());
        self.route_id(route_id.as_ref(), glob.path());
        self
    }

    fn route_id(&mut self, id: &str, glob: &[u8]) {
        let inner = self.mut_inner();
        let ref mut route_ids = inner.route_ids;

        match route_ids.get(id) {
            Some(other_glob) if glob != other_glob.as_bytes() => panic!("Duplicate route_id: {}", id),
            _ => ()
        };

        route_ids.insert(id.to_string(), String::from_utf8_lossy(glob).to_string());
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Get, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Post, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Put, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Delete, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Head, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Patch, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Options, glob, handler, route_id)
    }

    /// Route will match any method, including gibberish.
    /// In case of ambiguity, handlers specific to methods will be preferred.
    pub fn any<G, H, I, S, T>(&mut self, glob: G, handler: H, route_id: I) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              I: AsRef<str>,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        let glob = glob.into();
        let types = glob.types().map(|types| types.store());

        self.mut_inner().wildcard.push(Recognizer::new(glob.path(), Box::new(handler), types).unwrap());
        self.route_id(route_id.as_ref(), glob.path());
        self
    }

    fn recognize(&self, method: &Method, path: &str) -> Option<RouteMatch> {
        self.inner.routers
            .get(method)
            .and_then(|recognizers| recognizers.recognize(path))
            .or(self.inner.wildcard.recognize(path))
    }

    fn handle_options(&self, path: &str) -> Response {
        static METHODS: &'static [Method] = &[
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Delete,
            Method::Head,
            Method::Patch
        ];

        // Get all the available methods and return them.
        let mut options = vec![];

        for method in METHODS.iter() {
            self.inner.routers.get(method).map(|recognizers| {
                if let Some(_) = recognizers.recognize(path) {
                    options.push(method.clone());
                }
            });
        }
        // If GET is there, HEAD is also there.
        if options.contains(&Method::Get) && !options.contains(&Method::Head) {
            options.push(Method::Head);
        }

        let mut response = Response::new().with_status(StatusCode::Ok);
        response.headers.set(header::Allow(options));
        response
    }

    fn handle_method(&self, request: &mut Request) -> Option<FerrumResult<Response>> {
        if let Some(matched) = self.recognize(&request.method, request.uri.path()) {
            request.extensions.insert::<Router>(matched.params);
            request.extensions.insert::<RouterInner>(self.inner.clone());
            Some(matched.handler.handle(request))
        } else {
            None
        }
    }
}

impl Key for Router {
    type Value = Params;
}

impl Key for RouterInner {
    type Value = Arc<RouterInner>;
}

impl Handler for Router {
    fn handle(&self, request: &mut Request) -> FerrumResult<Response> {
        self.handle_method(request).unwrap_or_else(||
            match request.method {
                Method::Options => Ok(self.handle_options(request.uri.path())),
                // For HEAD, fall back to GET. Hyper ensures no response body is written.
                Method::Head => {
                    request.method = Method::Get;
                    self.handle_method(request).unwrap_or(
                        Err(
                            FerrumError::new(
                                NoRoute,
                                Some(Response::new()
                                    .with_status(StatusCode::NotFound))
                            )
                        )
                    )
                }
                _ => Err(
                    FerrumError::new(
                        NoRoute,
                        Some(Response::new()
                                .with_status(StatusCode::NotFound))
                    )
                )
            }
        )
    }
}

/// The error thrown by router if there is no matching route,
/// it is always accompanied by a NotFound response.
#[derive(Debug, PartialEq, Eq)]
pub struct NoRoute;

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("No matching route found.")
    }
}

impl Error for NoRoute {
    fn description(&self) -> &str { "No Route" }
}

/// The error thrown by router if a request was redirected
/// by adding or removing a trailing slash.
#[derive(Debug, PartialEq, Eq)]
pub struct TrailingSlash;

impl fmt::Display for TrailingSlash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("The request had a trailing slash.")
    }
}

impl Error for TrailingSlash {
    fn description(&self) -> &str { "Trailing Slash" }
}

#[cfg(test)]
mod test {
    use super::Router;
    use ferrum::{header, mime, Method, Request, Response};

    #[test]
    fn test_handle_options_post() {
        let mut router = Router::new();
        router.post("/", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        let resp = router.handle_options("/");
        let headers = resp.headers.get::<header::Allow>().unwrap();
        let expected = header::Allow(vec![Method::Post]);
        assert_eq!(&expected, headers);
    }

    #[test]
    fn test_handle_options_get_head() {
        let mut router = Router::new();
        router.get("/", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        let resp = router.handle_options("/");
        let headers = resp.headers.get::<header::Allow>().unwrap();
        let expected = header::Allow(vec![Method::Get, Method::Head]);
        assert_eq!(&expected, headers);
    }

    #[test]
    fn test_handle_any_ok() {
        let mut router = Router::new();
        router.post("/post", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        router.any("/post", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        router.put("/post", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        router.any("/get", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "any");

        assert!(router.recognize(&Method::Get, "/post").is_some());
        assert!(router.recognize(&Method::Get, "/get").is_some());
    }

    #[test]
    fn test_request() {
        let mut router = Router::new();
        router.post("/post", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        router.get("/post", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");

        assert!(router.recognize(&Method::Post, "/post").is_some());
        assert!(router.recognize(&Method::Get, "/post").is_some());
        assert!(router.recognize(&Method::Put, "/post").is_none());
        assert!(router.recognize(&Method::Get, "/post/").is_some());
        assert!(router.recognize(&Method::Post, "/post/").is_some());
    }

    #[test]
    fn test_not_found() {
        let mut router = Router::new();
        router.put("/put", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "");
        assert!(router.recognize(&Method::Patch, "/patch").is_none());
    }

    #[test]
    #[should_panic]
    fn test_same_route_id() {
        let mut router = Router::new();
        router.put("/put", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "my_route_id");
        router.get("/get", |_: &mut Request| {
            Ok(Response::new().with_content("", mime::TEXT_PLAIN))
        }, "my_route_id");
    }

    #[test]
    fn test_wildcard_regression() {
        let mut router = Router::new();
        router.options(".*", |_: &mut Request| Ok(Response::new().with_content("", mime::TEXT_PLAIN)), "id1");
        router.put("/upload/{filename}", |_: &mut Request| Ok(Response::new().with_content("", mime::TEXT_PLAIN)), "id2");
        assert!(router.recognize(&Method::Options, "/foo").is_some());
        assert!(router.recognize(&Method::Put, "/foo").is_none());
        assert!(router.recognize(&Method::Put, "/upload/foo").is_some());
    }
}
