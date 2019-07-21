use editor::{
    make_json_lang, make_keymap_lang, AstForest, Command, CommandGroup, Doc, TreeCmd, TreeNavCmd,
};
use language::LanguageSet;
// use pretty::{PlainText, PrettyDocument};

#[test]
fn test_keymap_lang() {
    let lang_set = LanguageSet::new();

    let (km_lang, km_note_set) = make_keymap_lang();
    lang_set.insert(km_lang.name().to_owned(), km_lang);
    let forest = AstForest::new(&lang_set);
    let km = lang_set.get("keymap").unwrap();
    // let km = &km_lang;

    let mut doc2 = Doc::new(
        "KeymapDoc",
        forest.new_fixed_tree(km, km.lookup_construct("root"), &km_note_set),
    );

    assert!(doc2.execute(CommandGroup::Group(vec![Command::TreeNav(
        TreeNavCmd::Child(0)
    ),])));

    assert!(
        doc2.execute(CommandGroup::Group(vec![Command::Tree(TreeCmd::Replace(
            // forest.new_flexible_tree(&km, km.lookup_construct("dict"), &km_note_set,)
            // forest.new_fixed_tree(&km, km.lookup_construct("entry"), &km_note_set,)
            forest.new_fixed_tree(&km, km.lookup_construct("entry"), &km_note_set)
        )),]))
    );

    // assert!(doc2.execute(CommandGroup::Group(vec![Command::Tree(
    //     TreeCmd::InsertPrepend(forest.new_flexible_tree(
    //         &km,
    //         km.lookup_construct("dict"),
    //         &km_note_set,
    //     ))
    // ),])));

    // assert!(doc2.execute(CommandGroup::Group(vec![Command::Tree(
    //     TreeCmd::InsertPrepend(forest.new_fixed_tree(
    //         &km,
    //         km.lookup_construct("entry"),
    //         &km_note_set,
    //     ))
    // ),])));
}

// #[test]
// fn test_json_and_keymap() {
//     let (lang_set, json_note_set) = make_json_lang();
//     let (km_lang, km_note_set) = make_keymap_lang();
//     lang_set.insert("keymap".into(), km_lang);

//     let forest = AstForest::new(&lang_set);
//     let json = lang_set.get("json").unwrap();
//     let km = lang_set.get("keymap").unwrap();

//     // let mut doc1 = Doc::new(
//     //     "JsonDoc",
//     //     forest.new_fixed_tree(json, json.lookup_construct("root"), &json_note_set),
//     // );

//     // assert!(doc1.execute(CommandGroup::Group(vec![Command::TreeNav(
//     //     TreeNavCmd::Child(0)
//     // ),])));

//     // assert!(doc1.execute(CommandGroup::Group(vec![
//     //     Command::Tree(TreeCmd::Replace(forest.new_flexible_tree(
//     //         &json,
//     //         json.lookup_construct("list"),
//     //         &json_note_set,
//     //     ))),
//     //     Command::Tree(TreeCmd::InsertPrepend(forest.new_fixed_tree(
//     //         &json,
//     //         json.lookup_construct("true"),
//     //         &json_note_set,
//     //     ))),
//     // ])));

//     let mut doc2 = Doc::new(
//         "KeymapDoc",
//         forest.new_fixed_tree(km, km.lookup_construct("root"), &km_note_set),
//     );

//     assert!(doc2.execute(CommandGroup::Group(vec![Command::TreeNav(
//         TreeNavCmd::Child(0)
//     ),])));

//     assert!(doc2.execute(CommandGroup::Group(vec![
//         Command::Tree(TreeCmd::Replace(forest.new_flexible_tree(
//             &km,
//             km.lookup_construct("dict"),
//             &json_note_set,
//         ))),
//         Command::Tree(TreeCmd::InsertPrepend(forest.new_fixed_tree(
//             &km,
//             km.lookup_construct("entry"),
//             &json_note_set,
//         ))),
//     ])));
// }
