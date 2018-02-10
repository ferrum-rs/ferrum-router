use std::collections::BTreeMap;

use regex::Regex;

use ferrum::{Request, Uri};
use ferrum::error::{HyperResult, HyperError};
use router::RouterInner;

pub trait UriFor {
    fn replace(&self, glob_regex: &Regex, params: BTreeMap<String, String>) -> HyperResult<Uri>;
}

impl UriFor for Uri {
    fn replace(&self, glob_regex: &Regex, mut params: BTreeMap<String, String>) -> HyperResult<Uri> {
        if self != "*" {
            let mut uri = String::new();

            if let Some(scheme) = self.scheme() {
                uri.push_str(scheme);
                uri.push_str("://");
            }

            if let Some(authority) = self.authority() {
                uri.push_str(authority);
            }

            let path = replace_regex_captures(self.path(), glob_regex, &mut params);

            if !path.is_empty() {
                uri.push_str(&path);
            }

            if !params.is_empty() {
                uri.push_str("?");
                let count = params.len();
                for (index, (ref key, ref value)) in params.into_iter().enumerate() {
                    uri.push_str(key);
                    uri.push_str("=");
                    uri.push_str(value);
                    if index < count - 1 {
                        uri.push_str("&");
                    }
                }
            }

            uri.parse().map_err(HyperError::from)
        } else {
            Ok(self.clone())
        }
    }
}

/// Generate a URI based off of the currently requested URI.
///
/// The `route_id` used during route registration will be used here again.
///
/// `params` will be inserted as route parameters if fitting, the rest will be appended as query
/// parameters.
pub fn uri_for(request: &Request, route_id: &str, params: BTreeMap<String, String>) -> Uri {
    let inner = request.extensions.get::<RouterInner>()
        .expect("Couldn\'t find router set up properly.");
    let glob_regex = inner.route_ids.get(route_id)
        .expect("No route with that ID");

    match request.uri.replace(glob_regex, params) {
        Ok(uri) => uri,
        Err(err) => panic!("New URI parse error: {:?}", err)
    }
}

pub fn replace_regex_captures(source: &str, regex: &Regex, params: &mut BTreeMap<String, String>) -> String {
    let mut replacements = vec![];

    if let Some(captures) = regex.captures(source) {
        for capture_name in regex.capture_names() {
            if let Some(name) = capture_name {
                if let Some(replacement) = params.remove(name) {
                    if let Some(capture_match) = captures.name(name) {
                        replacements.push((capture_match.start(), capture_match.end(), replacement));
                    }
                }
            }
        }
    }

    if !replacements.is_empty() {
        replacements.sort_by(|a, b| a.0.cmp(&b.0));

        let mut target = String::new();
        let mut index = 0;

        for (start, end, replacement) in replacements.into_iter() {
            let source_chunk = &source.as_bytes()[index .. start];
            target.push_str(&String::from_utf8_lossy(source_chunk));
            target.push_str(&replacement);
            index = end;
        }
        if index < source.len() {
            target.push_str(&String::from_utf8_lossy(&source.as_bytes()[index ..]));
        }

        target
    } else {
        source.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ferrum::Response;
    use recognizer::{Recognizer, DefaultStore};

    #[test]
    fn test_uri_replace() {
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
                "http://localhost/foo/bam/"
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
                vec![("id", "some")],
                "http://localhost/foo/",
                "http://localhost/foo/?id=some"
            ),
            (
                "/{controller}/{action}/{id:[0-9]*}",
                vec![("controller", "test"), ("action", "run"), ("id", "some")],
                "http://localhost/foo/",
                "http://localhost/foo/?action=run&controller=test&id=some"
            ),
            (
                "/{controller}/{action}/{id:[0-9]*}",
                vec![],
                "http://localhost/foo/bar/baz",
                "http://localhost/foo/bar/baz"
            ),
        ];

        for (pattern, replacements_and_params, source_uri, target_uri) in samples {
            let handler = Box::new(|_: &mut Request| { Ok(Response::new()) });
            let recognizer = Recognizer::new(pattern, handler, Option::<&DefaultStore>::default()).unwrap();

            let uri: Uri = source_uri.parse().unwrap();
            let uri = uri.replace(&recognizer.glob_regex, {
                let mut params = BTreeMap::new();
                for (key, value) in replacements_and_params {
                    params.insert(key.into(), value.into());
                }
                params
            }).unwrap();
            assert_eq!(target_uri, uri);
        }
    }

    #[test]
    fn test_replace_regex_captures() {
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
            let mut params = BTreeMap::new();
            for (key, value) in replacements {
                params.insert(key.into(), value.into());
            }

            let result = replace_regex_captures(source, &regex, &mut params);
            assert_eq!(target, result);
        }
    }
}
