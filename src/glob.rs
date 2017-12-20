use recognizer::types::{DefaultType, Types, TypeName, TypePattern};

#[derive(Default)]
pub struct Glob<S = DefaultType, N = DefaultType, P = DefaultType>
    where S: AsRef<[u8]>,
          N: TypeName,
          P: TypePattern
{
    pub prefix: S,
    pub postfix: S,
    pub types: Types<N, P>
}

impl<N, P> Glob<DefaultType, N, P>
    where N: TypeName + Default,
          P: TypePattern + Default
{
    pub fn without_suffixes() -> Glob<DefaultType, N, P> {
        Glob {
            prefix: DefaultType::default(),
            postfix: DefaultType::default(),
            types: Types::default()
        }
    }
}

impl<S, N, P> Glob<S, N, P>
    where S: AsRef<[u8]>,
          N: TypeName + Default,
          P: TypePattern + Default
{
    pub fn produce<T>(&self, path: T) -> Vec<u8>
        where T: AsRef<[u8]>
    {
        let mut glob = Vec::new();
        glob.extend_from_slice(self.prefix.as_ref());


        let mut index = 0;
        let mut iter = path.as_ref().iter();
        while let Some(&bch) = iter.next() {
            match bch {
                b'{' => {
                    ;
                },
                _ => ()
            }
            index += 1;
        }

        glob.extend_from_slice(self.postfix.as_ref());
        glob
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_glob() {
        // Glob with all default parameters
        let glob = <Glob>::default();

        assert_eq!(glob.prefix, "");
        assert_eq!(glob.postfix, "");
        assert_eq!(glob.types.0, Types::<DefaultType, DefaultType>::default().0);

        // Default Glob for assigned parameters
        let mut glob = Glob::default();
        glob.prefix = "^";
        glob.types.insert("key", "value");

        let value = glob.types.get("key").unwrap();
        assert_eq!(*value, "value");
        assert_eq!(glob.prefix, "^");
        assert_eq!(glob.postfix, "");

        // Glob with String suffixes
        let mut glob = Glob {
            prefix: "^".to_string(),
            ..Glob::default()
        };
        glob.types.insert("key", "value");

        let value = glob.types.get("key").unwrap();
        assert_eq!(*value, "value");
        assert_eq!(glob.prefix, "^".to_string());
        assert_eq!(glob.postfix, "");


        let mut types = Types::default();
        types.insert("key", "value".to_string());

        // Glob with types and default suffixes
        let glob = Glob {
            types,
            ..Glob::without_suffixes()
        };

        let value = glob.types.get("key").unwrap();
        assert_eq!(*value, "value".to_string());
        assert_eq!(glob.prefix, "");
        assert_eq!(glob.postfix, "");


        let mut types = Types::default();
        types.insert("key".to_string(), "value".to_string());

        // Glob with types and default suffixes
        let mut glob = Glob::without_suffixes();
        glob.types = types;

        let value = glob.types.get("key").unwrap();
        assert_eq!(*value, "value".to_string());
        assert_eq!(glob.prefix, "");
        assert_eq!(glob.postfix, "");
    }
}