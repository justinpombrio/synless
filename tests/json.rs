use std::fs;
use std::path::Path;
use synless::{parsing::JsonParser, DocName, Engine, Settings};

const JSON_PATH: &str = "data/json_lang.ron";

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
}
