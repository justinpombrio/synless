use std::fs;
use std::path::Path;
use synless::{DocName, Engine, Location, Node, Settings, Storage};

const KEYHINT_PATH: &str = "data/keyhints_lang.ron";
const KEYHINT_NOTATION_NAME: &str = "DefaultDisplay";

#[test]
fn test_keyhint_lang() {
    let mut engine = Engine::new(Settings::default());

    let keyhint_lang_ron = fs::read_to_string(KEYHINT_PATH).unwrap();
    let language_name = engine
        .load_language_ron(Path::new(KEYHINT_PATH), &keyhint_lang_ron)
        .unwrap();
    engine
        .set_source_notation(&language_name, KEYHINT_NOTATION_NAME)
        .unwrap();

    let doc_name = DocName::Auxilliary("<testing>".to_owned());
    engine.add_empty_doc(&doc_name, &language_name).unwrap();

    let mut cursor = engine.get_doc(&doc_name).unwrap().cursor();
    let s = engine.raw_storage_mut();
    let lang = s.language(&language_name).unwrap();

    let c_key = lang.construct(s, "Key").unwrap();
    let c_hint = lang.construct(s, "Hint").unwrap();
    let c_entry = lang.construct(s, "Entry").unwrap();

    let add_entry = |s: &mut Storage, cursor: &mut Location, key: &str, hint: &str| {
        let key_node = Node::with_text(s, c_key, key.to_owned()).unwrap();
        let hint_node = Node::with_text(s, c_hint, hint.to_owned()).unwrap();
        let entry_node = Node::with_children(s, c_entry, [key_node, hint_node]).unwrap();
        cursor.insert(s, entry_node).unwrap();
        *cursor = cursor.next_sibling(s).unwrap();
    };

    add_entry(s, &mut cursor, "h", "left");
    add_entry(s, &mut cursor, "l", "right");

    let output = engine.print_source(&doc_name).unwrap();
    let expected = "h left\nl right";
    assert_eq!(output, expected);
}
