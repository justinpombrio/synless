use crossterm::execute;
use crossterm::style::{
    Attribute, Attributes, Color, ResetColor, SetAttributes, SetBackgroundColor, SetForegroundColor,
};
use std::io::stdout;

fn colors() -> [Color; 7] {
    use Color::*;

    [White, Red, Yellow, Green, Cyan, Blue, Magenta]
}

fn letters() -> [char; 7] {
    ['W', 'R', 'Y', 'G', 'C', 'B', 'M']
}

fn attribute_combos() -> Vec<(Attributes, String)> {
    let attributes_and_names = [
        (Attribute::Reverse, "reversed"),
        (Attribute::Bold, "bold"),
        (Attribute::Dim, "dim"),
        (Attribute::Italic, "italic"),
        (Attribute::Underlined, "underlined"),
    ];

    let mut combos = vec![(Attributes::default(), Vec::new())];
    for (attribute, name) in attributes_and_names {
        let mut new_combos = Vec::new();
        for (mut attributes, mut names) in combos {
            new_combos.push((attributes, names.clone()));
            attributes = attributes | attribute;
            names.push(name.to_owned());
            new_combos.push((attributes, names));
        }
        combos = new_combos;
    }

    combos
        .into_iter()
        .map(|(attributes, names)| {
            if names.is_empty() {
                (attributes, "regular".to_owned())
            } else {
                (attributes, names.join(","))
            }
        })
        .collect::<Vec<_>>()
}

fn rainbow(attributes: Attributes, label: &str, set_fg: bool, set_bg: bool) {
    let letters = letters();
    let colors = colors();

    execute!(stdout(), SetAttributes(attributes)).unwrap();
    for i in 0..7 {
        let letter = letters[i];
        let color = colors[i];
        if set_fg {
            print!("{}", SetForegroundColor(color));
        }
        if set_bg {
            print!("{}", SetBackgroundColor(color));
        }
        print!("{}", letter);
    }
    println!("{}{} {}", Attribute::Reset, ResetColor, label);
}

fn main() {
    for (attributes, name) in attribute_combos() {
        rainbow(attributes, &name, true, false);
    }
}
