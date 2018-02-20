
# Synless

    I think that I shall never see
    A poem lovely as a tree.
    - Joyce Kilmer

Synless is a code editor that is not a text editor[1].

------

## The Status Quo

Code is normally saved as, and edited as, text. Why is that?

Text is a _wildly_ successful format. If you look at any domain in
computing you'll see text everywhere. Data representation? CSV, JSON,
and XML are all built on top of text. The internet? Html, urls, and
email. Operating systems? The Unix philosophy says proclaims: "Write
programs to handle text streams, because that is a universal
interface (Doug McIlroy)." And so it's no surprise that programming
languages too almost always have a textual syntax.

You will notice something in common with all of these examples: they
are not _plain_ text. Instead, each of them (and the syntax of almost
every programming language) is a specific format built _on top of_
text. In fact, it often makes sense to talk about a language
_independently of its textual representation_. For example,
[in rust](https://docs.serde.rs/serde_json/), JSON values can be
defined like this:

    enum Value {
        Null,
        Bool(bool),
        Number(Number),
        String(String),
        Array(Vec<Value>),
        Object(Map<String, Value>),
    }
    
Notice that this is _not_ the [JSON standard](http://json.org/). The
JSON standard mentions square brackets, and this does not. Instead,
this is another standard _implicitly_ defined by the JSON standard.
We now have two ways to talk about JSON: as text, or as a `Value`.
Likewise, editors have a choice: they can treat JSON more like text,
or they can treat it more like a `Value`. In practice, editors tend to
treat documents as text:

1. They _save and load_ documents as text.
2. They _internally represent_ documents as text (or something
   functionally equivalent to text such as a gap buffer or rope).
3. They present a _textual editing interface_: they have a cursor with
   a line and column, and you can insert or delete a character at any
   position.

This has a clear advantage: it lets you edit a document even if your
editor has no idea what it is. Sure, you won't get syntax
highlighting, or code completion, or jump-to-defintion, or sugar or
spice or anything nice. But at least you can _open_ the thing, and
carefully make a few changes.

There's a downside, though. Since your editor represents documents as
text, when you _do_ have features like syntax highlighting or code
completion, they must be hacked together with regexps and hope. But
[regexps aren't parsers](FILL), and hope quickly leaves when you're
writing emacs regexps to match perl regexps and you're not sure how
[all the backslashes got there but you think they're multiplying](https://github.com/jrockway/cperl-mode/blob/master/cperl-mode.el#L8224).

The point I'm trying to make is that _if syntax highlighting, code
completion, jump-to-definition, etc. are slow or buggy, it's probably
because they have to deal with the editor representing the document as
text_. I feel confident making this assetion because these problems
become easy if you have a fully structured representation of the
document.

[FILL]

<!-- and hope quickly leaves after you
somehow get Paredit into a state where your lisp has an odd number of
parens, and now Paredit is dutifully enforcing that your parens never
again be balanced. -->

[1] I stole this phrase. Here's the original: [https://www.facebook.com/notes/kent-beck/prune-a-code-editor-that-is-not-a-text-editor/1012061842160013](Prune: A code editor that is not a text editor).

------

# Scratch Notes

    {"Question": "Is JSON text?", "Answer": [true, false]}

    Object
      'Question' -> "Is JSON text?"
      'Answer'   -> Array
                      true
                      false

See c2: the power of text.

Cons of binary:

- difficult to understand, test, debug
- usually implies proprietary

Cons of text:

- no special editor support needed
- version control
- share files between languages and applications easily
- read, debug, and edit with common tools
- easier to port

TODO: List aspects of editor: trees, layouts, etc.
TODO: Mention exceptions: sexprs, Forth, paredit
