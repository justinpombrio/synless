# Synless

Synless is a code editor that is not a text editor. It is under development.

![A screenshot of a JSON document open in Synless. The cursor is on a JSON
object in the document, rather than being on a text
character.](doc/screenshots/synless.png)

**Navigate and edit code as a tree.** To delete an element of a list, you move
your cursor onto it and hit backspace. Contrast this to text editors where you
must also delete the comma after the list element... unless it's the first
element of the list in which case you have to delete the comma after it...
unless it's also the _only_ element in which case you don't delete any comma.
_We have been editing code at the wrong level of abstraction._

**No syntax errors.** You can't end up with (e.g.) mismatched brackets because
there's no mechanism to insert or delete a single bracket. It's not a thing you
can put your cursor on. The brackets don't even appear _in memory_: while a
document is being edited, Synless stores it as a tree, not as text.

**Context menu (WIP).** Since Synless knows the syntax of the language you're
editing, it can show you what's allowed syntactically to go where. (There
currently is a context menu, but it isn't smart about filtering the kinds of
nodes you're allowed to insert.)

**Leave holes in your code.** Synless supports _holes_, which represent a piece
of code that hasn't yet been written. This is essential for practical editing.

**Reformat on the fly.** Synless pretty prints the part of the document you're
viewing every time it redraws the screen, so if you resize the window it will
reformat the code to fit instead of needing to hard-wrap. It does so using the
dramatically over-engineered [partial pretty
printing](https://github.com/justinpombrio/partial-pretty-printer) library.

**Language Agnostic (WIP).** Synless is written to be able to support editing
multiple languages. Adding a language requires giving a parser, pretty printer,
and tree grammar for it. Currently the only supported language is JSON, and
there are tricky UI questions around making a good editing experience for larger
languages, but this is the goal we've been working towards since the beginning.

**Customize your language's syntax.** Synless is so named because it's about
removing the centrality of syntax from the way that we think about languages.
When editing a JavaScript file (not yet supported), if you think the keyword
`function` is too long you ought to have the power to change its display
notation and make it say `fn` instead.

**Languages without parsers (WIP).** One of the goals of Synless it to support
the creation of languages that don't have parsers. It will be able to store a
document's tree strucure directly in a format that's generic and trivially
machine parsable (perhaps s-expressions, or JSON, or a binary format). This way
the language needs only a pretty printer (for displaying the code in Synless),
not a parser, and pretty printers are much easier to write.

**Scriptable.** Synless is scriptable and customizable in
[Rhai](https://rhai.rs/).

## Usage

1. If you don't have Rust, [install it](https://rust-lang.org/tools/install/).
2. Navigate to the `synless/` directory and run `cargo run --release`.
3. Press "space" then "o" to open or create a JSON file.

There's no tutorial yet, but the context menu on the right will show you the
actions you can perform at any point.

**Synless is in alpha. It may eat your files. Please open an issue if it does.**

## Links

[Why Synless? And why "Synless"?](doc/why.md)

[An Incomplete Survey of Tree Editors](doc/survey.md)
