#![feature(slice_patterns)]

mod common;

use common::{assert_strings_eq, make_json_doc};

// TODO: test horz concat

#[test]
fn test_lay_out_json_80() {
    let doc = make_json_doc();

    assert_strings_eq(
        &doc.write(80),
        r#"{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address":
    {
      "streetAddress": "21 2nd Street",
      "city": "New York",
      "state": "NY",
      "postalCode": "10021-3100"
    },
  "phoneNumbers":
    [{
       "type": "home",
       "number": "212 555-1234"
     },
     "surprise string!",
     {
       "type": "office",
       "number": "646 555-4567"
     }],
  "children": [],
  "spouse": null,
  "favoriteThings":
    ["raindrops on roses", "whiskers on kittens", {"color": "red"},
     {"food": "pizza"}, "brown paper packages", [{}]],
  "lists": ["first", ["second", ["third", ["fourth", "fifth is longer"]]]]
}"#,
    )
}

#[test]
fn test_lay_out_json_30() {
    let doc = make_json_doc();

    assert_strings_eq(
        &doc.write(30),
        r#"{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address":
    {
      "streetAddress":
        "21 2nd Street",
      "city": "New York",
      "state": "NY",
      "postalCode":
        "10021-3100"
    },
  "phoneNumbers":
    [{
       "type": "home",
       "number":
         "212 555-1234"
     },
     "surprise string!",
     {
       "type": "office",
       "number":
         "646 555-4567"
     }],
  "children": [],
  "spouse": null,
  "favoriteThings":
    ["raindrops on roses",
     "whiskers on kittens",
     {"color": "red"},
     {"food": "pizza"},
     "brown paper packages",
     [{}]],
  "lists":
    ["first",
     ["second",
      ["third",
       ["fourth",
        "fifth is longer"]]]]
}"#,
    );
}

#[test]
#[should_panic]
fn test_lay_out_json_28() {
    // The doc won't fit in 28 characters
    // Eventually the strings should wrap, so it
    let doc = make_json_doc();
    doc.write(28);
}
