use super::*;

#[test]
fn parse_glob_direct() {
    let types = Store::<String, String>::default();
    let (regex, params) = Recognizer::parse_glob("", &types).unwrap();

    assert!(!regex.is_match("test"));
    assert!(regex.is_match(""));
    assert!(regex.is_match("/"));
    assert!(!regex.is_match("//"));
    assert_eq!(params, Vec::<ParamChunk>::new());

    let (regex, params) = Recognizer::parse_glob("/", &types).unwrap();

    assert!(!regex.is_match("test"));
    assert!(!regex.is_match(""));
    assert!(regex.is_match("/"));
    assert!(!regex.is_match("//"));
    assert_eq!(params, Vec::<ParamChunk>::new());

    let (regex, params) = Recognizer::parse_glob("/posts/new", &types).unwrap();

    assert!(!regex.is_match(""));
    assert!(!regex.is_match("test"));
    assert!(!regex.is_match("/"));
    assert!(regex.is_match("/posts/new"));
    assert!(regex.is_match("/posts/new/"));
    assert!(!regex.is_match("/posts/new//"));
    assert!(!regex.is_match("/posts/new/test"));
    assert_eq!(params, Vec::<ParamChunk>::new());

    let (regex, params) = Recognizer::parse_glob("/posts/new", &types).unwrap();

    assert!(!regex.is_match(""));
    assert!(!regex.is_match("test"));
    assert!(!regex.is_match("/"));
    assert!(regex.is_match("/posts/new"));
    assert!(regex.is_match("/posts/new/"));
    assert!(!regex.is_match("/posts/new//"));
    assert!(!regex.is_match("/posts/new/test"));
    assert_eq!(params, Vec::<ParamChunk>::new());
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
    assert_eq!(params, vec![ParamChunk { name: "name".to_string(), start: 7, end: 13 }]);

    let (regex, params) = Recognizer::parse_glob("/posts/{tail:.*}", &types).unwrap();

    assert!(!regex.is_match(""));
    assert!(!regex.is_match("test"));
    assert!(!regex.is_match("/"));
    assert!(!regex.is_match("/posts"));
    assert!(regex.is_match("/posts/"));
    assert!(regex.is_match("/posts//"));
    assert!(regex.is_match("/posts/new"));
    assert!(regex.is_match("/posts/new/"));
    assert!(regex.is_match("/posts/new/test"));
    assert!(regex.is_match("/posts/new/test/"));
    assert_eq!(params, vec![ParamChunk { name: "tail".to_string(), start: 7, end: 16 }]);

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
        assert_eq!(params, vec![ParamChunk { name: "id".to_string(), start: 7, end: glob.len() }]);
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