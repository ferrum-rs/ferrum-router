use recognizer::types::{GlobTypes, DefaultStore};

#[derive(Default)]
pub struct Glob<S, T = DefaultStore>
    where S: AsRef<[u8]>,
          T: GlobTypes,
{
    path: S,
    types: Option<T>,
}

impl<S, T> Glob<S, T>
    where S: AsRef<[u8]>,
          T: GlobTypes,
{
    pub fn new(path: S, types: Option<T>) -> Self {
        Glob {
            path,
            types,
        }
    }

    pub fn path(&self) -> &[u8] {
        self.path.as_ref()
    }

    pub fn types(&self) -> Option<&T> {
        self.types.as_ref()
    }
}

impl<S> From<S> for Glob<S, DefaultStore>
    where S: AsRef<[u8]>
{
    fn from(path: S) -> Self {
        Glob::new(path, None)
    }
}

impl<S, T> From<(S, T)> for Glob<S, T>
    where S: AsRef<[u8]>,
          T: GlobTypes,
{
    fn from(pair: (S, T)) -> Self {
        let (path, types) = pair;
        Glob::new(path, Some(types))
    }
}


#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use super::*;
    use recognizer::types::Store;

    fn assert_glob_key<'a, G, S, T>(glob: G, key: <T as GlobTypes>::Name, expected: <T as GlobTypes>::Pattern)
        where G: Into<Glob<S, T>>,
              S: AsRef<[u8]> + 'a,
              T: GlobTypes + 'a,
    {
        let glob = glob.into();
        let types = glob.types().unwrap().store();
        let value = types.get(key.borrow()).unwrap();
        assert_eq!(value.as_ref(), expected.as_ref());
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


        let types_default = DefaultStore::default();
        let types_string = Store::<String, String>::default();

        let glob: Glob<_, &DefaultStore> = Glob::from((glob_str, &types_default));
        assert_eq!(glob.path(), glob_str.as_bytes());
        assert!(glob.types().is_some());
        assert_eq!(glob.types().unwrap().store(), &types_default);

        let glob = Glob::from((glob_str, &types_string));
        assert_eq!(glob.path(), glob_str.as_bytes());
        assert!(glob.types().is_some());
        let types = glob.types().unwrap();
        assert_eq!(types.store(), &types_string);

        let mut types = Store::<&str, String>::default();
        types.insert("key", "value".to_string());
        assert_glob_key(("", &types), "key", "value".to_string());
        assert_glob_key(("", types), "key", "value".to_string());
    }
}