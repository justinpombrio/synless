mod doc;
mod json_notation;

pub use doc::Doc;
use json_notation::make_json_notation;

/// If the strings aren't equal, print them with better formatting than the assert_eq!() macro and then panic.
pub fn assert_strings_eq(left: &str, right: &str) {
    if left != right {
        eprintln!("left string:\n{}", left);
        eprintln!("\nright string:\n{}", right);
        panic!("strings are not equal");
    }
}

pub fn make_json_doc() -> Doc {
    let notations = make_json_notation();

    let string = |text: &str| -> Doc { Doc::new_leaf(notations["string"].clone(), text) };
    let int = |num: usize| -> Doc { Doc::new_leaf(notations["number"].clone(), &num.to_string()) };
    let null = || -> Doc { Doc::new_branch(notations["null"].clone(), Vec::new()) };
    let boolean = |value: bool| -> Doc {
        let name = if value { "true" } else { "false" };
        Doc::new_branch(notations[name].clone(), Vec::new())
    };
    let list = |elems: Vec<Doc>| -> Doc { Doc::new_branch(notations["list"].clone(), elems) };
    let dict_entry = |key_and_val: (&str, Doc)| -> Doc {
        let key = Doc::new_leaf(notations["string"].clone(), key_and_val.0);
        Doc::new_branch(notations["dict_entry"].clone(), vec![key, key_and_val.1])
    };
    let dict = |entries: Vec<(&str, Doc)>| -> Doc {
        Doc::new_branch(
            notations["dict"].clone(),
            entries.into_iter().map(dict_entry).collect(),
        )
    };

    dict(vec![
        ("firstName", string("John")),
        ("lastName", string("Smith")),
        ("isAlive", boolean(true)),
        ("age", int(27)),
        (
            "address",
            dict(vec![
                ("streetAddress", string("21 2nd Street")),
                ("city", string("New York")),
                ("state", string("NY")),
                ("postalCode", string("10021-3100")),
            ]),
        ),
        (
            "phoneNumbers",
            list(vec![
                dict(vec![
                    ("type", string("home")),
                    ("number", string("212 555-1234")),
                ]),
                string("surprise string!"),
                dict(vec![
                    ("type", string("office")),
                    ("number", string("646 555-4567")),
                ]),
            ]),
        ),
        ("children", list(vec![])),
        ("spouse", null()),
        (
            "favoriteThings",
            list(vec![
                string("raindrops on roses"),
                string("whiskers on kittens"),
                dict(vec![("color", string("red"))]),
                dict(vec![("food", string("pizza"))]),
                string("brown paper packages"),
                list(vec![dict(vec![])]),
            ]),
        ),
        (
            "lists",
            list(vec![
                string("first"),
                list(vec![
                    string("second"),
                    list(vec![
                        string("third"),
                        list(vec![string("fourth"), string("fifth is longer")]),
                    ]),
                ]),
            ]),
        ),
    ])
}

pub fn make_long_json_list() -> Doc {
    let notations = make_json_notation();
    let boolean = |value: bool| -> Doc {
        let name = if value { "true" } else { "false" };
        Doc::new_branch(notations[name].clone(), Vec::new())
    };
    let list = |elems: Vec<Doc>| -> Doc { Doc::new_branch(notations["list"].clone(), elems) };

    list(vec![
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(true),
        boolean(false),
        boolean(true),
        boolean(false),
    ])
}
