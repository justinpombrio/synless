mod doc;
mod example_notation;

pub use doc::Doc;
pub use example_notation::example_notation;

pub fn make_example_doc() -> Doc {
    let notations = example_notation();

    let leaf = |construct: &str, contents: &str| -> Doc {
        let note = notations.get(construct).unwrap().clone();
        Doc::new_leaf(note, contents)
    };

    let branch = |construct: &str, children: Vec<Doc>| -> Doc {
        let note = notations.get(construct).unwrap().clone();
        Doc::new_branch(note, children)
    };

    branch(
        "function",
        vec![
            leaf("id", "foo"),
            branch("args", vec![leaf("id", "abc"), leaf("id", "def")]),
            branch(
                "add",
                vec![leaf("string", "abcdef"), leaf("string", "abcdef")],
            ),
        ],
    )
}
