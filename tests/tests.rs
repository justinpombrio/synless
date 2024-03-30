use partial_pretty_printer as ppp;
use synless::{
    AritySpec, ConstructSpec, DocRef, GrammarSpec, LanguageSpec, Location, Node, NotationSetSpec,
    SortSpec, Storage,
};

// e.g. example.com?p1=v1,p2=v2,p3,p4=v4
fn urllang() -> LanguageSpec {
    use ppp::notation_constructors::{
        child, count, empty, fold, indent, left, lit, nl, right, Count, Fold,
    };

    LanguageSpec {
        name: "urllang".to_owned(),
        grammar: GrammarSpec {
            constructs: vec![
                ConstructSpec {
                    name: "String".to_owned(),
                    arity: AritySpec::Texty,
                    is_comment_or_ws: false,
                    key: Some('s'),
                },
                ConstructSpec {
                    name: "Equals".to_owned(),
                    arity: AritySpec::Fixed(vec![
                        SortSpec(vec!["String".to_owned()]),
                        SortSpec(vec!["String".to_owned()]),
                    ]),
                    is_comment_or_ws: false,
                    key: Some('='),
                },
                ConstructSpec {
                    name: "Params".to_owned(),
                    arity: AritySpec::Listy(SortSpec(vec!["param".to_owned()])),
                    is_comment_or_ws: false,
                    key: None,
                },
                ConstructSpec {
                    name: "Url".to_owned(),
                    arity: AritySpec::Fixed(vec![
                        SortSpec(vec!["String".to_owned()]),
                        SortSpec(vec!["Params".to_owned()]),
                    ]),
                    is_comment_or_ws: false,
                    key: None,
                },
            ],
            sorts: vec![(
                "param".to_owned(),
                SortSpec(vec!["String".to_owned(), "Equals".to_owned()]),
            )],
            root_sort: SortSpec(vec!["Url".to_owned()]),
        },
        default_notation_set: NotationSetSpec {
            name: "Testlang_notation".to_owned(),
            notations: vec![
                ("String".to_owned(), child(0) + child(1)),
                ("Equals".to_owned(), child(0) + lit("=") + child(1)),
                ("Url".to_owned(), child(0) + child(1)),
                (
                    "Params".to_owned(),
                    indent(
                        "    ",
                        None,
                        count(Count {
                            zero: empty(),
                            one: lit("?") + child(0),
                            many: (lit("?")
                                + fold(Fold {
                                    first: child(0),
                                    join: left() + lit("&") + right(),
                                }))
                                | lit("?")
                                    ^ fold(Fold {
                                        first: child(0),
                                        join: left() + nl() + lit("&") + right(),
                                    }),
                        }),
                    ),
                ),
            ],
        },
    }
}

fn node_with_text(s: &mut Storage, lang_name: &str, construct_name: &str, text: &str) -> Node {
    let lang = s.get_language(lang_name).unwrap();
    let construct = lang.get_construct(s, construct_name).unwrap();
    Node::with_text(s, construct, text.to_owned()).unwrap()
}

fn node_with_children(
    s: &mut Storage,
    lang_name: &str,
    construct_name: &str,
    children: impl IntoIterator<Item = Node>,
) -> Node {
    let lang = s.get_language(lang_name).unwrap();
    let construct = lang.get_construct(s, construct_name).unwrap();
    Node::with_children(s, construct, children).unwrap()
}

#[test]
fn test_doc_ref() {
    let mut s = Storage::new();
    s.add_language(urllang()).unwrap();

    // example.com?param1=val1,param2=val2,done
    let domain = node_with_text(&mut s, "urllang", "String", "example.com");
    let param_1 = node_with_text(&mut s, "urllang", "String", "param1");
    let val_1 = node_with_text(&mut s, "urllang", "String", "val1");
    let eq_1 = node_with_children(&mut s, "urllang", "Equals", [param_1, val_1]);
    let param_2 = node_with_text(&mut s, "urllang", "String", "param2");
    let val_2 = node_with_text(&mut s, "urllang", "String", "val2");
    let eq_2 = node_with_children(&mut s, "urllang", "Equals", [param_2, val_2]);
    let done = node_with_text(&mut s, "urllang", "String", "done");
    let params = node_with_children(&mut s, "urllang", "Params", [eq_1, eq_2, done]);
    let url = node_with_children(&mut s, "urllang", "Url", [domain, params]);

    let doc_ref = DocRef::new(&s, Location::after(&s, url), url);

    let actual = match ppp::pretty_print_to_string(doc_ref, 80) {
        Ok(actual) => actual,
        Err(err) => panic!("{}", err),
    };
    let expected = "example.com?param1=val1&param2=val2&done";
    assert_eq!(actual, expected);

    let actual = match ppp::pretty_print_to_string(doc_ref, 20) {
        Ok(actual) => actual,
        Err(err) => panic!("{}", err),
    };
    let expected = "example.com?\n    param1=val1\n    &param2=val2\n    &done";
    assert_eq!(actual, expected);
}
