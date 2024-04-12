use std::fs;
use std::path::Path;
use synless::{DocName, Engine, Location, Node, Settings, Storage};

const SELECTION_PATH: &str = "data/selection_lang.ron";
const SELECTION_NOTATION_NAME: &str = "DefaultDisplay";

#[test]
fn test_selection_lang() {
    let mut engine = Engine::new(Settings::default());

    let selection_lang_ron = fs::read_to_string(SELECTION_PATH).unwrap();
    let language_name = engine
        .load_language_ron(Path::new(SELECTION_PATH), &selection_lang_ron)
        .unwrap();
    engine
        .set_source_notation(&language_name, SELECTION_NOTATION_NAME)
        .unwrap();

    let doc_name = DocName::Auxilliary("<testing>".to_owned());
    engine.add_empty_doc(&doc_name, &language_name).unwrap();

    let mut cursor = engine.get_doc(&doc_name).unwrap().cursor();
    let s = engine.raw_storage_mut();
    let lang = s.language(&language_name).unwrap();

    let add_elem = |s: &mut Storage, cursor: &mut Location, construct_name: &str, text: &str| {
        let construct = lang.construct(s, construct_name).unwrap();
        let node = Node::with_text(s, construct, text.to_owned()).unwrap();
        cursor.insert(s, node).unwrap();
    };

    add_elem(s, &mut cursor, "Input", "oo");
    add_elem(s, &mut cursor, "Custom", "oo");
    add_elem(s, &mut cursor, "Literal", "foobar.rs");
    add_elem(s, &mut cursor, "NonLiteral", "..");
    add_elem(s, &mut cursor, "Literal", "baz.rs");

    let output = engine.print_source(&doc_name).unwrap();
    let expected = "> oo\n[+] oo\nfoobar.rs\n..\nbaz.rs";
    assert_eq!(output, expected);
}
