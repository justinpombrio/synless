use std::fs;
use std::path::Path;
use synless::{parsing::JsonParser, DocName, Engine, Settings, Node};
use synless::command::{TreeEdCommand, TreeNavCommand};

const JSON_PATH: &str = "data/json_lang.ron";

fn new_node(engine: &mut Engine, lang_name: &str, construct_name: &str) -> Node {
    let s = engine.raw_storage_mut();
    let lang = s.language(lang_name).unwrap();
    let construct = lang.construct(s, construct_name).unwrap();
    Node::new(s, construct)
}

#[test]
fn test_json() {
    let mut engine = Engine::new(Settings::default());

    let json_lang_ron = fs::read_to_string(JSON_PATH).unwrap();
    let language_name = engine
        .load_language_ron(Path::new(JSON_PATH), &json_lang_ron)
        .unwrap();
    engine.add_parser(&language_name, JsonParser);

    let doc_name = DocName::Auxilliary("<testing>".to_owned());
    let source = "{\"primitives\": [true, false, null, 5.3, \"string!\"]}";
    engine
        .load_doc_from_source(doc_name.clone(), &language_name, source)
        .unwrap();
    let output = engine.print_source(&doc_name).unwrap();
    assert_eq!(output, source);

    // Regression test for https://github.com/justinpombrio/synless/issues/123
    // (memory leak when failing to insert new node).
    engine.set_visible_doc(&doc_name).unwrap();
    engine.execute(TreeNavCommand::FirstChild).unwrap();
    let node = new_node(&mut engine, &language_name, "True");
    engine.execute(TreeEdCommand::Insert(node)).unwrap_err();
    drop(engine);
}
