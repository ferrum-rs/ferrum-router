use std::borrow::Borrow;
use std::marker::PhantomData;
use recognizer::types::{DefaultTypes, NameDefaultType, PatternDefaultType, Types, TypeName, TypePattern};

#[derive(Default)]
pub struct Glob<S, N = NameDefaultType, P = PatternDefaultType, T = DefaultTypes>
    where T: Borrow<Types<N, P>>,
          S: AsRef<[u8]>,
          N: TypeName,
          P: TypePattern
{
    path: S,
    types: Option<T>,
    phantom_n: PhantomData<N>,
    phantom_p: PhantomData<P>,
}

impl<S, N, P, T> Glob<S, N, P, T>
    where T: Borrow<Types<N, P>>,
          S: AsRef<[u8]>,
          N: TypeName,
          P: TypePattern
{
    fn new(path: S, types: Option<T>) -> Self {
        Glob {
            path,
            types,
            phantom_n: PhantomData,
            phantom_p: PhantomData
        }
    }

    fn path(&self) -> &[u8] {
        self.path.as_ref()
    }

    fn types(&self) -> Option<&T> {
        self.types.as_ref()
    }
}

impl<S> From<S> for Glob<S, NameDefaultType, PatternDefaultType, DefaultTypes>
    where S: AsRef<[u8]>
{
    fn from(path: S) -> Self {
        Glob::new(path, None)
    }
}

impl<S, N, P, T> From<(S, T)> for Glob<S, N, P, T>
    where S: AsRef<[u8]>,
          N: TypeName,
          P: TypePattern,
          T: Borrow<Types<N, P>>
{
    fn from(pair: (S, T)) -> Self {
        let (path, types) = pair;
        Glob::new(path, Some(types))
    }
}


#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::cmp::PartialEq;
    use super::*;

    fn assert_glob_key<'a, G, S, N, P, T>(glob: G, key: N, expected: P)
        where G: Into<Glob<S, N, P, T>>,
              S: AsRef<[u8]> + 'a,
              N: TypeName + 'a,
              P: TypePattern + Debug + PartialEq + 'a,
              T: Borrow<Types<N, P>> + 'a
    {
        let glob = glob.into();
        let types = glob.types().unwrap().borrow();
        assert_eq!(*types.get(key.borrow()).unwrap(), expected);
    }

    #[test]
    fn glob_from() {
        let glob_str = "path/str";
        let glob = Glob::from(glob_str);
        assert_eq!(glob.path(), glob_str.as_bytes());
        assert!(glob.types().is_none());

        let glob_string = "path/string".to_string();
        let glob = Glob::from(glob_string.clone());
        assert_eq!(glob.path(), glob_string.as_bytes());
        assert!(glob.types().is_none());

        let glob_bytes = "path/bytes".as_bytes();
        let glob = Glob::from(glob_bytes);
        assert_eq!(glob.path(), glob_bytes);
        assert!(glob.types().is_none());

        let glob_vec = "path/vec".as_bytes().to_vec();
        let glob = Glob::from(glob_vec.clone());
        assert_eq!(glob.path(), glob_vec.as_slice());
        assert!(glob.types().is_none());


        let types_default = DefaultTypes::default();
        let types_string = Types::<String, String>::default();

        let glob: Glob<_, _, _, &DefaultTypes> = Glob::from((glob_str, &types_default));
        assert_eq!(glob.path(), glob_str.as_bytes());
        assert!(glob.types().is_some());
        assert_eq!(&glob.types().unwrap().0, &types_default.0);

        let glob = Glob::from((glob_str, &types_string));
        assert_eq!(glob.path(), glob_str.as_bytes());
        assert!(glob.types().is_some());
        let types = glob.types().unwrap();
        assert_eq!(&types.0, &types_string.0);

        let mut types = Types::<&str, String>::default();
        types.insert("key", "value".to_string());
        assert_glob_key(("", &types), "key", "value".to_string());
        assert_glob_key(("", types), "key", "value".to_string());
    }
}