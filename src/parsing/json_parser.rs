use super::{Parse, ParseError};
use crate::language::{Language, Storage};
use crate::tree::Node;
use crate::util::{bug_assert, error, SynlessBug, SynlessError};
use partial_pretty_printer as ppp;

#[derive(Debug)]
pub struct JsonParser;

impl Parse for JsonParser {
    fn name(&self) -> &str {
        "BuiltinJsonParser"
    }

    fn parse(
        &mut self,
        s: &mut Storage,
        file_name: &str,
        source: &str,
    ) -> Result<Node, SynlessError> {
        // Serde json uses 1-indexed positions; we use 0-indexed positions.
        let json = serde_json::from_str(source).map_err(|err| ParseError {
            pos: Some(ppp::Pos {
                row: (err.line() as ppp::Row).saturating_sub(1),
                col: (err.column() as ppp::Col).saturating_sub(1),
            }),
            file_name: file_name.to_owned(),
            message: format!("{}", err),
        })?;

        let json_lang = s.language("Json")?;
        let json_node = json_to_node(s, json, json_lang).map_err(|construct| {
            error!(
                Parse,
                "Construct '{}' missing from JSON language spec", construct
            )
        })?;
        let root_node = Node::with_children(s, json_lang.root_construct(s), [json_node])
            .ok_or_else(|| error!(Parse, "Bug in Json Parser: root node arity mismatch"))?;
        Ok(root_node)
    }
}

fn json_to_node(
    s: &mut Storage,
    json: serde_json::Value,
    json_lang: Language,
) -> Result<Node, &'static str> {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};

    let make_node = |s: &mut Storage, construct_name: &'static str| -> Result<Node, &'static str> {
        let construct = json_lang
            .construct(s, construct_name)
            .ok_or(construct_name)?;
        Ok(Node::new(s, construct))
    };

    match json {
        Null => make_node(s, "Null"),
        Bool(false) => make_node(s, "False"),
        Bool(true) => make_node(s, "True"),
        String(string) => {
            let node = make_node(s, "String")?;
            node.text_mut(s).unwrap().set(string);
            Ok(node)
        }
        Number(n) => {
            let node = make_node(s, "Number")?;
            node.text_mut(s).unwrap().set(n.to_string());
            Ok(node)
        }
        Array(array) => {
            let node = make_node(s, "Array")?;
            for value in array {
                let child = json_to_node(s, value, json_lang)?;
                bug_assert!(
                    node.insert_last_child(s, child),
                    "Wrong arity in Json Array"
                );
            }
            Ok(node)
        }
        Object(object) => {
            let node = make_node(s, "Object")?;
            for (key, value) in object {
                let key_node = make_node(s, "String")?;
                key_node.text_mut(s).unwrap().set(key);
                let value_node = json_to_node(s, value, json_lang)?;
                let pair_construct = json_lang.construct(s, "ObjectPair").ok_or("ObjectPair")?;
                let child = Node::with_children(s, pair_construct, [key_node, value_node])
                    .bug_msg("Wrong arity in Json ObjectPair");
                bug_assert!(
                    node.insert_last_child(s, child),
                    "Wrong arity in Json Object"
                );
            }
            Ok(node)
        }
    }
}
