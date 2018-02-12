use regex::Regex;

use ferrum::{Request, Uri};
use ferrum::error::{HyperResult, HyperError};
use router::RouterInner;
use recognizer::{Recognizer, Params, ParamChunk};

pub trait UriFor {
    fn generate(&self, glob_path: Option<&str>, recognizer: &Recognizer, params: Params) -> HyperResult<Uri>;
}

impl UriFor for Uri {
    fn generate(&self, glob_path: Option<&str>, recognizer: &Recognizer, mut params: Params) -> HyperResult<Uri> {
        if self != "*" {
            let mut uri = String::new();

            if let Some(scheme) = self.scheme() {
                uri.push_str(scheme);
                uri.push_str("://");
            }

            if let Some(authority) = self.authority() {
                uri.push_str(authority);
            }

            let path = if let Some(path) = glob_path {
                generate_for_glob(path, recognizer, &mut params)
            } else {
                generate_for_regex_captures(self.path(), &recognizer.glob_regex, &mut params)
            };

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
pub fn uri_for(request: &Request, route_id: &str, params: Params) -> Uri {
    let inner = request.extensions.get::<RouterInner>()
        .expect("Couldn\'t find router set up properly.");
    let (ref glob_path, ref recognizer) = *inner.route_ids.get(route_id)
        .expect("No route with that ID");

    match request.uri.generate(Some(glob_path), recognizer, params) {
        Ok(uri) => uri,
        Err(err) => panic!("New URI parse error: {:?}", err)
    }
}

pub fn generate_for_glob(source: &str, recognizer: &Recognizer, params: &mut Params) -> String {
    let mut replacements = vec![];

    for &ParamChunk { ref name, start, end } in recognizer.param_chunks.iter() {
        if let Some(replacement) = params.remove(name) {
            replacements.push((start, end, replacement));
        }
    }
    replace_params(source, replacements)
}


pub fn generate_for_regex_captures(source: &str, regex: &Regex, params: &mut Params) -> String {
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
    replace_params(source, replacements)
}

fn replace_params(source: &str, mut replacements: Vec<(usize, usize, String)>) -> String {
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
mod tests;
