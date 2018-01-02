use std::error::Error;

use ferrum::Handler;
use regex::Regex;

pub mod types;
pub mod glob;
pub mod matcher;
pub use self::types::*;
pub use self::glob::*;
pub use self::matcher::*;

pub type RecognizerResult<T = Recognizer> = Result<T, Box<Error>>;

pub struct Recognizer {
    pub glob_regex: Regex,
    pub param_names: Vec<String>,
    pub handler: Box<Handler>,
}

pub trait Recognize {
    fn recognize<'a>(&'a self, path: &str) -> Option<RouteMatch<'a>>;
}

impl Recognizer {
    pub fn new<G, N, P>(glob: G, handler: Box<Handler>, types: Option<&Store<N, P>>) -> RecognizerResult
        where G: AsRef<[u8]>,
              N: TypeName,
              P: TypePattern
    {
        let types_default = DefaultStore::with_default_types();
        let (glob_regex, param_names) = match types {
            Some(types) => Recognizer::parse_glob(glob, types),
            None => Recognizer::parse_glob(glob, &types_default)
        }?;

        Ok(Recognizer {
            glob_regex,
            param_names,
            handler,
        })
    }

    pub fn parse_glob<G, N, P>(glob: G, types: &Store<N, P>) -> RecognizerResult<(Regex, Vec<String>)>
        where G: AsRef<[u8]>,
              N: TypeName,
              P: TypePattern
    {
        let mut param_names = Vec::<String>::new();
        let mut pattern = "^".as_bytes().to_vec();

        let identifier_regex = Regex::new("^[_a-zA-Z][_0-9a-zA-Z]*$").unwrap();

        let mut iter = glob.as_ref().iter();
        while let Some(&bch) = iter.next() {
            match bch {
                b'{' => {
                    let mut param_name = Vec::new();
                    let mut param_type = Vec::new();
                    let mut is_type = false;

                    while let Some(&bch) = iter.next() {
                        match bch {
                            b' ' | b'\t' | b'\r' | b'\n' => continue,
                            b':' if !is_type => is_type = true,
                            b'}' => {
                                if param_name.len() > 0 || param_type.len() > 0 {
                                    let param_name = String::from_utf8(param_name)?;

                                    let param_type = if param_type.len() > 0 {
                                        String::from_utf8(param_type)?
                                    } else {
                                        param_name.clone()
                                    };

                                    let regex_chunk = if param_name.len() > 0 && !identifier_regex.is_match(param_name.as_str()) {
                                        "{".to_string() + param_name.as_str() + "}"
                                    } else {
                                        let prefix = if param_name.len() > 0 {
                                            let prefix = format!("(?P<{}>", param_name);
                                            param_names.push(param_name);
                                            prefix
                                        } else {
                                            "(".to_string()
                                        };

                                        prefix + if let Some(regex_pattern) = types.get(param_type.as_str()) {
                                            regex_pattern.as_ref()
                                        } else {
                                            Type::STRING_PATTERN
                                        } + ")"
                                    };
                                    pattern.extend(regex_chunk.as_bytes().iter());
                                }
                                break;
                            },
                            _ if is_type => param_type.push(bch),
                            _ => param_name.push(bch),
                        }
                    }
                },
                _ => pattern.push(bch),
            }
        }
        let mut pattern = String::from_utf8(pattern)?;
        pattern += if pattern.chars().rev().next().unwrap_or('_') == '/' { "$" } else { "/?$" };
        Ok((Regex::new(&pattern)?, param_names))
    }
}

impl Recognize for Recognizer {
    fn recognize<'a>(&'a self, path: &str) -> Option<RouteMatch<'a>> {
        if let Some(captures) = self.glob_regex.captures(path) {
            let mut params = Params::new();
            for name in self.param_names.iter() {
                if let Some(param_match) = captures.name(name.as_str()) {
                    params.insert(name.clone(), param_match.as_str().to_string());
                }
            }
            Some(RouteMatch::new(&self.handler, params))
        } else {
            None
        }
    }
}

impl Recognize for Vec<Recognizer> {
    fn recognize<'a>(&'a self, path: &str) -> Option<RouteMatch<'a>> {
        for recognizer in self {
            if let Some(route_match) = recognizer.recognize(path) {
                return Some(route_match);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_glob_direct() {
        let types = Store::<String, String>::default();
        let (regex, params) = Recognizer::parse_glob("", &types).unwrap();

        assert!(!regex.is_match("test"));
        assert!(regex.is_match(""));
        assert!(regex.is_match("/"));
        assert!(!regex.is_match("//"));
        assert_eq!(params, Vec::<String>::new());

        let (regex, params) = Recognizer::parse_glob("/", &types).unwrap();

        assert!(!regex.is_match("test"));
        assert!(!regex.is_match(""));
        assert!(regex.is_match("/"));
        assert!(!regex.is_match("//"));
        assert_eq!(params, Vec::<String>::new());

        let (regex, params) = Recognizer::parse_glob("/posts/new", &types).unwrap();

        assert!(!regex.is_match(""));
        assert!(!regex.is_match("test"));
        assert!(!regex.is_match("/"));
        assert!(regex.is_match("/posts/new"));
        assert!(regex.is_match("/posts/new/"));
        assert!(!regex.is_match("/posts/new//"));
        assert!(!regex.is_match("/posts/new/test"));
        assert_eq!(params, Vec::<String>::new());
    }

    #[test]
    fn parse_glob_single_param() {
        let mut types = Store::default();
        let (regex, params) = Recognizer::parse_glob("/posts/{name}", &types).unwrap();

        assert!(!regex.is_match(""));
        assert!(!regex.is_match("test"));
        assert!(!regex.is_match("/"));
        assert!(regex.is_match("/posts/12"));
        assert!(regex.is_match("/posts/12/"));
        assert!(!regex.is_match("/posts/12/test"));
        assert!(regex.is_match("/posts/new"));
        assert!(regex.is_match("/posts/new/"));
        assert!(!regex.is_match("/posts/new/test"));
        assert_eq!(params, vec!["name".to_string()]);

        let globs = vec![
            "/posts/{id}",
            "/posts/{id:number}",
            "/posts/{ id: number }",
            "/posts/{ id:   number  }",
        ];
        types.insert("id", "[0-9]+");
        types.insert("number", Type::NUMBER_PATTERN);

        for glob in globs {
            let (regex, params) = Recognizer::parse_glob(glob, &types).unwrap();

            assert!(!regex.is_match(""), glob);
            assert!(!regex.is_match("test"), glob);
            assert!(!regex.is_match("/"), glob);
            assert!(regex.is_match("/posts/12"), glob);
            assert!(regex.is_match("/posts/12/"), glob);
            assert!(!regex.is_match("/posts/12a"), glob);
            assert!(!regex.is_match("/posts/12/test"), glob);
            assert!(!regex.is_match("/posts/new"), glob);
            assert!(!regex.is_match("/posts/new/"), glob);
            assert!(!regex.is_match("/posts/new/test"), glob);
            assert_eq!(params, vec!["id".to_string()]);
        }
    }

    #[cfg(all(test, feature = "nightly"))]
    mod benches {
        extern crate test;

        use super::*;

        #[bench]
        fn parse_glob_benchmark(bencher: &mut test::Bencher) {
            let mut types = Store::default();
            types.insert("number", "[0-9]+");

            bencher.iter(|| {
                Recognizer::parse_glob("/posts/{id:number}", &types).unwrap()
            });
        }
    }
}