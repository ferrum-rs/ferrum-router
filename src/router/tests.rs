use super::*;

use ferrum::{header, mime, Method, Request, Response};
use recognizer::{DefaultStore, DefaultStoreBuild, Type};

#[test]
fn test_handle_options_post() {
    let mut router = Router::new();
    router.post("/", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);

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
    }, None);

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
    }, None);
    router.any("/post", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);
    router.put("/post", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);
    router.any("/get", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, Some("any".into()));

    assert!(router.recognize(&Method::Get, "/post").is_some());
    assert!(router.recognize(&Method::Get, "/get").is_some());
}

#[test]
fn test_request() {
    let mut router = Router::new();
    router.post("/post", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);
    router.get("/post", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);

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
    }, None);

    assert!(router.recognize(&Method::Patch, "/patch").is_none());
}

#[test]
#[should_panic]
fn test_same_route_id() {
    let mut router = Router::new();
    router.put("/put", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, Some("my_route_id".into()));
    router.get("/get", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, Some("my_route_id".into()));
}

#[test]
fn test_wildcard_regression() {
    let mut router = Router::new();
    router.options(".*", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);
    router.put("/upload/{filename}", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);

    assert!(router.recognize(&Method::Options, "/foo").is_some());
    assert!(router.recognize(&Method::Put, "/foo").is_none());
    assert!(router.recognize(&Method::Put, "/upload/foo").is_some());
}

#[test]
fn test_glob_types() {
    let mut router = Router::new();
    let types = DefaultStore::with_default_types();

    router.get(".*", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);
    router.post("/upload/{filename}", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);
    router.post(("/send/{id:number}", &types), |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, None);

    assert!(router.recognize(&Method::Get, "/foo").is_some());
    assert!(router.recognize(&Method::Post, "/foo").is_none());
    assert!(router.recognize(&Method::Post, "/upload/foo").is_some());
    assert!(router.recognize(&Method::Post, "/send/12").is_some());
    assert!(router.recognize(&Method::Post, "/send/no").is_none());
}

#[test]
fn test_route_ids() {
    let mut router = Router::new();
    let types = DefaultStore::with_default_types();

    router.get(".*", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, Id::some("id1"));
    router.post("/upload/{filename}", |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, Id::some("id2"));
    router.post(("/send/{id:number}", &types), |_: &mut Request| {
        Ok(Response::new().with_content("", mime::TEXT_PLAIN))
    }, Id::some("id3"));

    let route_ids = &router.inner.route_ids;

    assert_eq!(3, route_ids.len());
    let (ref path, ref recognizer) = *route_ids.get("id1").unwrap();
    assert_eq!(".*", path);
    assert_eq!("^.*/?$", recognizer.glob_regex.as_str());

    let (ref path, ref recognizer) = *route_ids.get("id2").unwrap();
    assert_eq!("/upload/{filename}", path);
    assert_eq!(&format!("^/upload/(?P<filename>{})/?$", Type::STRING_PATTERN), recognizer.glob_regex.as_str());

    let (ref path, ref recognizer) = *route_ids.get("id3").unwrap();
    assert_eq!("/send/{id:number}", path);
    assert_eq!(&format!("^/send/(?P<id>{})/?$", Type::NUMBER_PATTERN), recognizer.glob_regex.as_str());
}
