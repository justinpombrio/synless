use std::fs;
use std::path::Path;
use synless::{parsing::JsonParser, Engine, Settings};

const JSON_PATH: &str = "data/json_lang.ron";
const JSON_NOTATION_NAME: &str = "DefaultDisplay";

#[test]
fn test_json() {
    let mut engine = Engine::new(Settings::default());

    let json_lang_ron = fs::read_to_string(JSON_PATH).unwrap();
    let language_name = engine.load_language_ron(JSON_PATH, &json_lang_ron).unwrap();
    engine.add_parser(&language_name, JsonParser).unwrap();
    engine
        .set_source_notation(&language_name, JSON_NOTATION_NAME)
        .unwrap();

    let doc_name = Path::new("<testing>");
    let source = "{\"primitives\": [true, false, null, 5.3, \"string!\"]}";
    engine
        .load_doc_from_source(doc_name, &language_name, source)
        .unwrap();
    let output = engine.print_source(doc_name).unwrap();
    assert_eq!(output, source);
}
