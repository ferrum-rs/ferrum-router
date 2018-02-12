use super::*;
use ferrum::Response;
use recognizer::{Recognizer, DefaultStore};

#[test]
fn test_uri_generate() {
    let samples = vec![
        (
            "/foo/{user:.+}",
            vec![("user", "bam")],
            "http://localhost/foo/bar/baz",
            "http://localhost/foo/bam"
        ),
        (
            "/foo/{user:.+}/",
            vec![("user", "bam")],
            "http://localhost/foo/bar/baz/",
            "http://localhost/foo/bam/"
        ),
        (
            "/foo/{user}",
            vec![("user", "bam")],
            "http://localhost/foo/bar/",
            "http://localhost/foo/bam"
        ),
        (
            "/{controller}/{action}/{id:[0-9]*}",
            vec![("controller", "test"), ("action", "run"), ("id", "some")],
            "http://localhost/foo/bar/5",
            "http://localhost/test/run/some"
        ),
        (
            "/{controller}/{action}/{id:[0-9]*}",
            vec![("controller", "test"), ("action", "run"), ("id", "some")],
            "http://localhost/foo/bar/",
            "http://localhost/test/run/some"
        ),
        (
            "/{controller}/{action}/{id:[0-9]*}",
            vec![("controller", "test"), ("action", "run"), ("id", "some"), ("query", "param")],
            "http://localhost/foo/bar/",
            "http://localhost/test/run/some?query=param"
        ),
        (
            "/{controller}/{action}/{id:[0-9]*}",
            vec![("id", "some")],
            "http://localhost/foo/",
            "http://localhost/{controller}/{action}/some"
        ),
        (
            "/{controller}/{action}/{id:[0-9]*}",
            vec![("controller", "test"), ("action", "run"), ("id", "some")],
            "http://localhost/foo/",
            "http://localhost/test/run/some"
        ),
        (
            "/{controller}/{action}/{id:[0-9]*}",
            vec![],
            "http://localhost/foo/bar/baz",
            "http://localhost/{controller}/{action}/{id:[0-9]*}"
        ),
    ];

    for (pattern, replacements_and_params, source_uri, target_uri) in samples {
        let handler = Box::new(|_: &mut Request| { Ok(Response::new()) });
        let recognizer = Recognizer::new(pattern, handler, Option::<&DefaultStore>::default()).unwrap();

        let uri: Uri = source_uri.parse().unwrap();
        let uri = uri.generate(Some(pattern), &recognizer, {
            let mut params = Params::new();
            for (key, value) in replacements_and_params {
                params.insert(key.into(), value.into());
            }
            params
        }).unwrap();
        assert_eq!(target_uri, uri);
    }
}

#[test]
fn test_generate_for_regex_captures() {
    let samples = vec![
        (
            "",
            vec![],
            "",
            ""
        ),
        (
            "",
            vec![],
            "test",
            "test"
        ),
        (
            "(?P<first>[\\w]+),\\s(?P<second>[0-9]*)",
            vec![],
            "values: november, 12",
            "values: november, 12"
        ),
        (
            "(?P<first>[\\w]+),\\s(?P<second>[0-9]*)",
            vec![("second", "42")],
            "values: november, 12",
            "values: november, 42"
        ),
        (
            "(?P<first>[\\w]+),\\s(?P<second>[0-9]*)",
            vec![("first", "test"), ("second", "42")],
            "values: november, 12",
            "values: test, 42"
        ),
        (
            "(?P<first>[\\w]+),\\s(?P<second>[0-9]*)",
            vec![("first", "test"), ("second", "42")],
            "values: november, ",
            "values: test, 42"
        ),
        (
            "(?P<first>[\\w]+),\\s(?P<second>[0-9]*)",
            vec![("first", "test"), ("second", "42")],
            "(values: november, 12).",
            "(values: test, 42)."
        ),
        (
            "^/upload/(?P<first>[\\w]+)/(?P<second>[0-9]+)/?$",
            vec![("first", "test"), ("second", "42")],
            "/upload/data/99",
            "/upload/test/42"
        ),
        (
            "^/upload/(?P<first>[\\w]+)/(?P<second>[0-9]+)/?$",
            vec![("first", "test"), ("second", "42")],
            "/upload/data/1/",
            "/upload/test/42/"
        ),
        (
            "^/upload/(?P<first>[\\w]+)/(?P<second>[0-9]+)/?$",
            vec![("first", "test"), ("second", "42")],
            "/upload/",
            "/upload/"
        ),
    ];

    for (pattern, replacements, source, target) in samples {
        let regex = Regex::new(pattern).unwrap();
        let mut params = Params::new();
        for (key, value) in replacements {
            params.insert(key.into(), value.into());
        }

        let result = generate_for_regex_captures(source, &regex, &mut params);
        assert_eq!(target, result);
    }
}