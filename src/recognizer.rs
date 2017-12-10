use std::collections::HashMap;
use std::error::Error;
use std::ops::{Deref, DerefMut};

use ferrum::Handler;
use regex::Regex;

pub type TypeCollection = HashMap<String, String>;

#[derive(Default)]
pub struct Types(pub TypeCollection);

impl Types {
    pub fn default_type() -> &'static str {
        "[^/.]+"
    }
}

impl Deref for Types {
    type Target = TypeCollection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Types {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type RecognizerResult<T = Recognizer> = Result<T, Box<Error>>;

pub struct Recognizer {
    pub glob_regex: Regex,
    pub param_names: Vec<String>,
    pub handler: Box<Handler>,
}

impl Recognizer {
    pub fn new<S>(glob: S, handler: Box<Handler>, types: &Types) -> RecognizerResult
        where S: AsRef<str>
    {
        let (glob_regex, param_names) = Recognizer::parse_glob(glob.as_ref(), types)?;
        Ok(Recognizer {
            glob_regex,
            param_names,
            handler,
        })
    }

    pub fn parse_glob(glob: &str, types: &Types) -> RecognizerResult<(Regex, Vec<String>)> {
        let mut param_names = Vec::<String>::new();
        let mut pattern = "^".as_bytes().to_vec();

        let mut iter = glob.as_bytes().iter();
        while let Some(&bch) = iter.next() {
            match bch {
                b'{' => {
                    let mut param_name = Vec::new();
                    let mut param_type = Vec::new();
                    let mut is_type = false;

                    while let Some(&bch) = iter.next() {
                        match bch {
                            b' ' | b'\t' | b'\r' | b'\n' => continue,
                            b':' => is_type = true,
                            b'}' => {
                                if param_name.len() > 0 || param_type.len() > 0 {
                                    let param_type = String::from_utf8(if param_type.len() > 0 {
                                        param_type.into()
                                    } else {
                                        param_name.clone()
                                    })?;

                                    let mut param_type_regex_string = String::from("(");
                                    if let Some(regex_string) = types.get(&param_type) {
                                        param_type_regex_string += regex_string;
                                    } else {
                                        param_type_regex_string += Types::default_type();
                                    }
                                    param_type_regex_string += ")";
                                    pattern.extend(param_type_regex_string.as_bytes().iter());

                                    if param_name.len() > 0 {
                                        param_names.push(String::from_utf8(param_name)?)
                                    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_glob_direct() {
        let types = Types::default();
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
    fn parse_glob_single_params() {
        let mut types = Types::default();
        let (regex, params) = Recognizer::parse_glob("/posts/{id}", &types).unwrap();

        assert!(!regex.is_match(""));
        assert!(!regex.is_match("test"));
        assert!(!regex.is_match("/"));
        assert!(regex.is_match("/posts/12"));
        assert!(regex.is_match("/posts/12/"));
        assert!(!regex.is_match("/posts/12/test"));
        assert!(regex.is_match("/posts/new"));
        assert!(regex.is_match("/posts/new/"));
        assert!(!regex.is_match("/posts/new/test"));
        assert_eq!(params, vec!["id".to_string()]);

        types.insert("id".to_string(), "[0-9]+".to_string());
        let (regex, params) = Recognizer::parse_glob("/posts/{id}", &types).unwrap();

        assert!(!regex.is_match(""));
        assert!(!regex.is_match("test"));
        assert!(!regex.is_match("/"));
        assert!(regex.is_match("/posts/12"));
        assert!(regex.is_match("/posts/12/"));
        assert!(!regex.is_match("/posts/12/test"));
        assert!(!regex.is_match("/posts/new"));
        assert!(!regex.is_match("/posts/new/"));
        assert!(!regex.is_match("/posts/new/test"));
        assert_eq!(params, vec!["id".to_string()]);
    }
}