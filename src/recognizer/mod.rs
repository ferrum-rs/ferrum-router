use std::error::Error;
use std::convert::AsRef;

use ferrum::Handler;
use regex::Regex;

pub mod types;
pub mod glob;
pub mod matcher;
pub use self::types::*;
pub use self::glob::*;
pub use self::matcher::*;

pub type RecognizerResult<T = Recognizer> = Result<T, Box<Error>>;

#[derive(Debug, PartialEq, Eq)]
pub struct ParamChunk {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

pub struct Recognizer {
    pub glob_regex: Regex,
    pub param_chunks: Vec<ParamChunk>,
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
        let (glob_regex, param_chunks) = match types {
            Some(types) => Recognizer::parse_glob(glob, types),
            None => Recognizer::parse_glob(glob, &types_default)
        }?;

        Ok(Recognizer {
            glob_regex,
            param_chunks,
            handler,
        })
    }

    pub fn parse_glob<G, N, P>(glob: G, types: &Store<N, P>) -> RecognizerResult<(Regex, Vec<ParamChunk>)>
        where G: AsRef<[u8]>,
              N: TypeName,
              P: TypePattern
    {
        let mut param_chunks = Vec::<ParamChunk>::new();
        let mut pattern = "^".as_bytes().to_vec();

        let identifier_regex = Regex::new("^[_a-zA-Z][_0-9a-zA-Z]*$").unwrap();

        let mut iter = glob.as_ref().iter().enumerate();
        while let Some((index, &bch)) = iter.next() {
            match bch {
                b'{' if index == 0 || glob.as_ref()[index - 1] != b'\\' => {
                    let mut param_name = Vec::new();
                    let mut param_type = Vec::new();
                    let mut is_type = false;
                    let start = index;

                    while let Some((index, &bch)) = iter.next() {
                        match bch {
                            b' ' | b'\t' | b'\r' | b'\n' => continue,
                            b':' if !is_type => is_type = true,
                            b'}' if index == 0 || glob.as_ref()[index - 1] != b'\\' => {
                                let end = index + 1;

                                if param_name.len() > 0 || param_type.len() > 0 {
                                    let param_name = String::from_utf8(param_name)?;

                                    let regex_chunk = if param_name.len() > 0 && !identifier_regex.is_match(param_name.as_str()) {
                                        "{".to_string() + param_name.as_str() + "}"
                                    } else {
                                        let prefix = if param_name.len() > 0 {
                                            let prefix = format!("(?P<{}>", param_name);
                                            param_chunks.push(ParamChunk {
                                                name: param_name.clone(),
                                                start,
                                                end
                                            });
                                            prefix
                                        } else {
                                            "(".to_string()
                                        };

                                        let param_type = String::from_utf8(param_type)?;

                                        let regex_type = if param_type.len() > 0 {
                                            if let Some(regex_pattern) = types.get(param_type.as_str()) {
                                                regex_pattern.as_ref()
                                            } else {
                                                param_type.as_str()
                                            }
                                        } else {
                                            if let Some(regex_pattern) = types.get(param_name.as_str()) {
                                                regex_pattern.as_ref()
                                            } else {
                                                Type::STRING_PATTERN
                                            }
                                        };

                                        prefix + regex_type + ")"
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
        Ok((Regex::new(&pattern)?, param_chunks))
    }
}

impl Recognize for Recognizer {
    fn recognize<'a>(&'a self, path: &str) -> Option<RouteMatch<'a>> {
        if let Some(captures) = self.glob_regex.captures(path) {
            let mut params = Params::new();
            for &ParamChunk { ref name, .. } in self.param_chunks.iter() {
                if let Some(param_match) = captures.name(name) {
                    params.insert(name.clone(), param_match.as_str().to_string());
                }
            }
            Some(RouteMatch::new(&self.handler, params))
        } else {
            None
        }
    }
}

impl<T> Recognize for Vec<T>
    where T: AsRef<Recognizer>
{
    fn recognize<'a>(&'a self, path: &str) -> Option<RouteMatch<'a>> {
        for recognizer in self {
            if let Some(route_match) = recognizer.as_ref().recognize(path) {
                return Some(route_match);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;
