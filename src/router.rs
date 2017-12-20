use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use ferrum::{Request, Response, Handler, FerrumResult, FerrumError};
use ferrum::{header, Method, StatusCode};
use ferrum::typemap::Key;

use recognizer::{Recognizer, Types, TypeName, TypePattern};

pub struct RouterInner<N, P>
    where N: TypeName,
          P: TypePattern
{
    /// The routers, specialized by method.
    pub routers: HashMap<Method, Vec<Recognizer>>,

    /// Routes that accept any method.
    pub wildcard: Vec<Recognizer>,

    /// Used in URL generation.
    pub route_ids: HashMap<String, String>,

    pub types: Types<N, P>,
}

/// `Router` provides an interface for creating complex routes as middleware
/// for the Ferrum framework.
pub struct Router<N, P>
    where N: TypeName,
          P: TypePattern
{
    inner: Arc<RouterInner<N, P>>
}

impl<N, P> Router<N, P>
    where N: TypeName,
          P: TypePattern
{
    /// Construct a new, empty `Router`.
    ///
    /// ```
    /// # use router::Router;
    /// let router = Router::new();
    /// ```
    pub fn new() -> Router<N, P> {
        Router {
            inner: Arc::new(RouterInner {
                routers: HashMap::new(),
                wildcard: Vec::new(),
                route_ids: HashMap::new(),
                types: Types(HashMap::new())
            })
        }
    }

    pub fn with_types(self, types: Types<N, P>) -> Router<N, P> {
        self.inner.types = types;
        self
    }

    fn mut_inner(&mut self) -> &mut RouterInner<N, P> {
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
    pub fn route<S, H, I>(&mut self, method: Method, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        // regex with capture groups
        // names of groups
        // values of groups
        // foo/{name}/{id:number} -> foo/([a-zA-Z_]+)/([0-9]+)
        // {name: <value>, id: <value>}

        self.mut_inner().routers
            .entry(method)
            .or_insert(Vec::new())
            .push(Recognizer::new(glob, Box::new(handler), &self.inner.types));
        self.route_id(route_id.as_ref(), glob.as_ref());
        self
    }

    fn route_id(&mut self, id: &str, glob: &str) {
        let inner = self.mut_inner();
        let ref mut route_ids = inner.route_ids;

        match route_ids.get(id) {
            Some(other_glob) if glob != other_glob => panic!("Duplicate route_id: {}", id),
            _ => ()
        };

        route_ids.insert(id.to_owned(), glob.to_owned());
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Get, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Post, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Put, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Delete, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Head, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Patch, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.route(Method::Options, glob, handler, route_id)
    }

    /// Route will match any method, including gibberish.
    /// In case of ambiguity, handlers specific to methods will be preferred.
    pub fn any<S, H, I>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router<N, P>
        where S: AsRef<str>,
              H: Handler,
              I: AsRef<str>
    {
        self.mut_inner().wildcard.add(glob.as_ref(), Box::new(handler));
        self.route_id(route_id.as_ref(), glob.as_ref());
        self
    }

    fn recognize(&self, method: &Method, path: &str)
                     -> Option<Match<&Box<Handler>>> {
        self.inner.routers.get(method).and_then(|router| router.recognize(path).ok())
            .or(self.inner.wildcard.recognize(path).ok())
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
            self.inner.routers.get(method).map(|router| {
                if let Some(_) = router.recognize(path).ok() {
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

    // Tests for a match by adding or removing a trailing slash.
    fn redirect_slash(&self, request : &Request) -> Option<FerrumError> {
        let mut uri = request.uri.clone();
        let mut path: Vec<&str> = uri.path().split("/").collect();

        if let Some(last_char) = path.chars().last() {
            {
                let mut path_segments = uri.as_mut().path_segments_mut().unwrap();
                if last_char == '/' {
                    // We didn't recognize anything without a trailing slash; try again with one appended.
                    path.pop();
                    path_segments.pop();
                } else {
                    // We didn't recognize anything with a trailing slash; try again without it.
                    path.push('/');
                    path_segments.push("");
                }
            }
        }

        self.recognize(&request.method, &path).and(Some(
            FerrumError::new(
                TrailingSlash,
                Response::new_redirect(url)
                    .with_status(StatusCode::MovedPermanently)
            )
        ))
    }

    fn handle_method(&self, request: &mut Request, path: &str) -> Option<FerrumResult<Response>> {
        if let Some(matched) = self.recognize(&request.method, path) {
            request.extensions.insert::<Router<N, P>>(matched.params);
            request.extensions.insert::<RouterInner<N, P>>(self.inner.clone());
            Some(matched.handler.handle(request))
        } else {
            self.redirect_slash(request)
                .and_then(|redirect| Some(Err(redirect)))
        }
    }
}

impl<N, P> Key for Router<N, P>
    where N: TypeName + 'static,
          P: TypePattern + 'static
{
    type Value = Params;
}

impl<N, P> Key for RouterInner<N, P>
    where N: TypeName + 'static,
          P: TypePattern + 'static
{
    type Value = Arc<RouterInner<N, P>>;
}

impl<N, P> Handler for Router<N, P>
    where N: TypeName + 'static,
          P: TypePattern + 'static
{
    fn handle(&self, request: &mut Request) -> FerrumResult<Response> {
        let path = request.url.path().join("/");

        self.handle_method(request, &path).unwrap_or_else(||
            match request.method {
                Method::Options => Ok(self.handle_options(&path)),
                // For HEAD, fall back to GET. Hyper ensures no response body is written.
                Method::Head => {
                    request.method = Method::Get;
                    self.handle_method(request, &path).unwrap_or(
                        Err(FerrumError::new(NoRoute, StatusCode::NotFound))
                    )
                }
                _ => Err(FerrumError::new(NoRoute, StatusCode::NotFound))
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
        assert!(router.recognize(&Method::Get, "/post/").is_none());
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
        router.options("*", |_: &mut Request| Ok(Response::new().with_content("", mime::TEXT_PLAIN)), "id1");
        router.put("/upload/*filename", |_: &mut Request| Ok(Response::new().with_content("", mime::TEXT_PLAIN)), "id2");
        assert!(router.recognize(&Method::Options, "/foo").is_some());
        assert!(router.recognize(&Method::Put, "/foo").is_none());
        assert!(router.recognize(&Method::Put, "/upload/foo").is_some());
    }
}
