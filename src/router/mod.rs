use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use ferrum::{Request, Response, Handler, FerrumResult, FerrumError};
use ferrum::{header, Method, StatusCode};
use ferrum::typemap::Key;

use recognizer::{Glob, GlobTypes, Recognizer, Recognize, RouteMatch, Params};

pub mod id;
pub use self::id::*;

pub struct RouterInner {
    /// The routers, specialized by method.
    pub routers: HashMap<Method, Vec<Arc<Recognizer>>>,

    /// Routes that accept any method.
    pub wildcard: Vec<Arc<Recognizer>>,

    /// Used in URI generation.
    pub route_ids: HashMap<Id, (String, Arc<Recognizer>)>,
}

/// `Router` provides an interface for creating complex routes as middleware
/// for the Ferrum framework.
pub struct Router {
    inner: Arc<RouterInner>
}

impl Router {
    /// Construct a new, empty `Router`.
    ///
    /// ```
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
    /// `route` supports glob patterns based on the rust regex and uses `{name}` (`{name: typename}`,
    /// `{name: pattern}`) for matching storing named segment of the request url in the `Params`
    /// object, which is stored in the request `extensions`.
    ///
    /// For instance, to route `Get` requests on any route matching
    /// `/users/{userid:[0-9]+}/{friendid:[0-9]+}` and store `userid` and `friend` in
    /// the exposed Params object:
    ///
    /// ```ignore
    /// let mut router = Router::new();
    /// router.route(Method::Get, "/users/{userid:[0-9]+}/{friendid:[0-9]+}", controller, "user_friend");
    /// ```
    ///
    /// `route_id` is a optional unique name for your route, and is used when generating an URI with
    /// `url_for`.
    ///
    /// The controller provided to route can be any `Handler`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Handler`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<G, H, S, T>(&mut self, method: Method, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        let glob = glob.into();
        let types = glob.types().map(|types| types.store());
        let recognizer = Arc::new(
            Recognizer::new(glob.path(), Box::new(handler), types).unwrap()
        );

        if let Some(route_id) = route_id {
            self.route_id(route_id, glob.path(), recognizer.clone());
        }

        self.mut_inner().routers
            .entry(method)
            .or_insert(Vec::new())
            .push(recognizer);
        self
    }

    fn route_id(&mut self, id: Id, glob_path: &[u8], recognizer: Arc<Recognizer>) {
        let inner = self.mut_inner();
        let ref mut route_ids = inner.route_ids;

        match route_ids.get(&id) {
            Some(&(ref other_glob_path, _)) if glob_path != other_glob_path.as_bytes() =>
                panic!("Duplicate route_id: {}", id),
            _ => ()
        };

        route_ids.insert(id, (String::from_utf8_lossy(glob_path).to_string(), recognizer));
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Get, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Post, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Put, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Delete, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Head, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Patch, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        self.route(Method::Options, glob, handler, route_id)
    }

    /// Route will match any method, including gibberish.
    /// In case of ambiguity, handlers specific to methods will be preferred.
    pub fn any<G, H, S, T>(&mut self, glob: G, handler: H, route_id: Option<Id>) -> &mut Router
        where G: Into<Glob<S, T>>,
              H: Handler,
              S: AsRef<[u8]>,
              T: GlobTypes,
    {
        let glob = glob.into();
        let types = glob.types().map(|types| types.store());
        let recognizer = Arc::new(
            Recognizer::new(glob.path(), Box::new(handler), types).unwrap()
        );

        if let Some(route_id) = route_id {
            self.route_id(route_id, glob.path(), recognizer.clone());
        }

        self.mut_inner().wildcard.push(recognizer);
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

#[cfg(test)]
mod tests;
