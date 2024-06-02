use crate::util::SynlessBug;
use regex::{self, Regex, RegexBuilder};

/// Return only the `items` that match the `input` search string, with the best matches first.
/// `get_str` returns a string representation for each item (to be compared with `input`).
pub fn fuzzy_search<T>(input: &str, items: Vec<T>, get_str: impl Fn(&T) -> &str) -> Vec<T> {
    if input.is_empty() {
        return items;
    }
    let searcher = Searcher::new(input).bug_msg("fuzzy_search: bad regex construction");
    let mut scored_items = items
        .into_iter()
        .filter_map(|item| searcher.score(get_str(&item)).map(|score| (score, item)))
        .collect::<Vec<_>>();
    scored_items.sort_by_key(|&(score, _)| score);
    scored_items
        .into_iter()
        .map(|(_, item)| item)
        .collect::<Vec<_>>()
}

/// Searches using regexes. Searching for `Baz` will return, in priority order:
///
/// ```text
///     Baz
///     baZ
///     Baz.js
///     baZ.js
///     FooBaz
///     foObaZ
///     FooBaz.js
///     foObaZ.js
/// ```
struct Searcher(Vec<Regex>);

impl Searcher {
    fn new(input: &str) -> Result<Searcher, regex::Error> {
        let pattern = input
            .split(" ")
            .map(regex::escape)
            .collect::<Vec<_>>()
            .join(".*");
        let regex_strs = vec![
            format!("^{}$", pattern),
            format!("^{}", pattern),
            format!("{}$", pattern),
            pattern.clone(),
        ];
        let mut regexes = Vec::new();
        for regex_str in regex_strs {
            regexes.push(Regex::new(&regex_str)?);
            regexes.push(
                RegexBuilder::new(&regex_str)
                    .case_insensitive(true)
                    .build()?,
            );
        }
        Ok(Searcher(regexes))
    }

    /// A score between for how well `item` matches the input used to construct `regexes`, where
    /// 0.0 is a perfect match and larger numbers are worse matches. Assumes that `regexes` are in
    /// order from most specific (good match) to least specific (bad match).
    fn score(&self, item: &str) -> Option<[usize; 3]> {
        for (i, regex) in self.0.iter().enumerate() {
            if let Some(matched) = regex.find(item) {
                return Some([i, matched.start(), item.len()]);
            }
        }
        None
    }
}

#[test]
fn test_fuzzy_search() {
    let items = vec!["foo", "bar", "foobarz"];
    let sorted = fuzzy_search("ba", items, |x| x);
    assert_eq!(sorted, vec!["bar", "foobarz"]);

    let items = vec!["foobarz", "bar", "foo"];
    let sorted = fuzzy_search("fo", items, |x| x);
    assert_eq!(sorted, vec!["foo", "foobarz"]);

    let items = vec![
        "bare.js",
        "foobear.js",
        "foobare.js",
        "foo.js",
        "foo.rs",
        "xfoobear.js",
        "fooo",
        "foo",
        "fo",
        "bar.rs.rs",
    ];

    let sorted = fuzzy_search("foo", items.clone(), |x| x);
    assert_eq!(
        sorted,
        vec![
            "foo",
            "fooo",
            "foo.js",
            "foo.rs",
            "foobear.js",
            "foobare.js",
            "xfoobear.js",
        ]
    );

    let sorted = fuzzy_search("", items, |x| x);
    assert_eq!(
        sorted,
        vec![
            "bare.js",
            "foobear.js",
            "foobare.js",
            "foo.js",
            "foo.rs",
            "xfoobear.js",
            "fooo",
            "foo",
            "fo",
            "bar.rs.rs",
        ]
    );

    assert_eq!(
        fuzzy_search("json", vec!["foo.json", "json.cpp"], |x| x),
        vec!["json.cpp", "foo.json"]
    );
}
