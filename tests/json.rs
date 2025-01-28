use std::fs;
use std::path::Path;
use synless::{parsing::JsonParser, DocName, Engine, Settings, Node, SynlessError};
use synless::command::{TreeEdCommand, TreeNavCommand};

const JSON_PATH: &str = "data/json_lang.ron";

struct JsonTester {
    engine: Engine,
    language_name: String,
    doc_name: DocName,
}

impl JsonTester {
    fn new() -> JsonTester {
        let mut engine = Engine::new(Settings::default());
        let json_lang_ron = fs::read_to_string(JSON_PATH).unwrap();
        let language_name = engine
            .load_language_ron(Path::new(JSON_PATH), &json_lang_ron)
            .unwrap();
        engine.add_parser(&language_name, JsonParser::new());
        let doc_name = DocName::Auxilliary("<testing>".to_owned());
        JsonTester {
            engine,
            language_name,
            doc_name,
        }
    }

    fn load_doc_from_source(&mut self, source: &str) -> Result<(), SynlessError> {
        self.engine.load_doc_from_source(self.doc_name.clone(), &self.language_name, source)?;
        self.engine.set_visible_doc(&self.doc_name)?;
        Ok(())
    }

    fn new_node(&mut self, construct_name: &str) -> Node {
        let s = self.engine.raw_storage_mut();
        let lang = s.language(&self.language_name).unwrap();
        let construct = lang.construct(s, construct_name).unwrap();
        Node::new(s, construct)
    }

    /// Assert that `source` parses, and when printed produces the same string.
    #[track_caller]
    fn assert_ok(&mut self, source: &str) {
        let result = self.load_doc_from_source(source);
        match result {
            Ok(()) => {
                let output = self.engine.print_source(&self.doc_name).unwrap();
                assert_eq!(output, source);
                self.engine.delete_doc(&self.doc_name).unwrap();
            }
            Err(error) => {
                eprintln!("{}", error);
                panic!("Error while parsing JSON");
            }
        }
    }

    /// Assert that `source` fails to parse, with an error message that contains `expected_error`
    /// as a substring.
    #[track_caller]
    fn assert_err(&mut self, source: &str, expected_error: &str) {
        let result = self.load_doc_from_source(source);
        let actual_error = match result {
            Ok(_) => panic!("Expected error"),
            Err(err) => err.to_string(),
        };
        if !actual_error.contains(expected_error) {
            eprintln!("EXPECTED ERROR:");
            eprintln!("{}", expected_error);
            eprintln!("ACTUAL ERROR:");
            eprintln!("{}", actual_error);
            eprintln!("END");
            panic!("Wrong error message");
        }
    }
}

#[test]
fn test_json() {
    let mut tester = JsonTester::new();

    // Primitives
    tester.assert_ok("[true, false, null, -5.3e11, \"string\"]");

    // String escapes
    // TODO: JSON printing doesn't currently escape!
    // tester.assert_ok(r#""escaped\\string""#);

    // Arrays
    tester.assert_ok("[[], [[]], [1], [1, 2], [1, 2, 3]]");
    tester.assert_err("]", "Extra ']'");
    tester.assert_err("[", "Array not closed");
    tester.assert_err("[5", "Array not closed");
    tester.assert_err("[5,", "Array not closed");
    tester.assert_err("[5,]", "Trailing comma");

    // Objects
    tester.assert_ok(r#"{"empty": {}, "one": {"1": 1}, "two": {"1": 1, "2": 2}}"#);
    tester.assert_err("}", "Extra '}'");
    tester.assert_err("{", "Object not closed");
    tester.assert_err("{\"key\":5", "Object not closed");
    tester.assert_err("{\"key\":5,", "Object not closed");
    tester.assert_err("{\"key\":5,}", "Trailing comma");
    tester.assert_err("{5", "expected string");
    tester.assert_err("{\"key\" 5", "expected ':'");
}


#[test]
fn test_regression_123() {
    // Regression test for https://github.com/justinpombrio/synless/issues/123
    // (memory leak when failing to insert new node).
    let mut tester = JsonTester::new();
    let source = "{\"primitives\": [true, false, null, 5.3, \"string!\"]}";
    tester.load_doc_from_source(source).unwrap();
    tester.engine.execute(TreeNavCommand::FirstChild).unwrap();
    let node = tester.new_node("True");
    tester.engine.execute(TreeEdCommand::Insert(node)).unwrap_err();
    drop(tester);
}
