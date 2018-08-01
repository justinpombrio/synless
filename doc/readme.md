
# Synless

    I think that I shall never see
    A poem lovely as a tree.
    - Joyce Kilmer

Synless is a code editor that is not a text editor[1].

**This whole project is in a pre-alpha stage. Even the design
  documents are under construction at best.***

[TODO: The important thing to convey here is my perspective, and my
frustration of seeing ASTs but editing text.]

------

## The Status Quo

Code is normally saved as, and edited as, text. Why is that?

Text is a _wildly_ successful format. If you look at any domain in
computing you'll see text everywhere:

- Data representation? CSV, JSON, and XML are all built on top of text.
- The internet? Html, urls, and email.
- Operating systems? The Unix Philosophy proclaims: "Write
programs to handle text streams, because that is a universal interface
(Doug McIlroy)."

And so it's no surprise that programming languages almost always have
a textual syntax.

You will notice something in common with all of these examples, though: they
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
We now have two ways to talk about JSON: as text, and as a `Value`.
Likewise, editors have a choice: they can treat JSON more like text,
or they can treat it more like a `Value`. In practice, editors tend to
treat documents as text:

1. They _save and load_ documents as text.
2. They _internally represent_ documents as text (or something
   functionally equivalent to text such as a gap buffer or rope).
3. They present a _textual editing interface_: they have a cursor with
   a line and column, and you can insert or delete a character at any
   position.

This has advantages and disadvantages.

### Advantages of Text

Essentially all of the advantages of text stem from the fact that it
is, as Doug McIlroy says, a "universal interface".

When a document is represented as text, you can edit it even if your
editor has no idea what kind of document it is. Sure, you won't get
syntax highlighting, or code completion, or jump-to-defintion, or any
of those other nice features. But at least you can _open_ the thing,
and carefully make a few changes.

Furthermore, you know exactly what the editing interface is going to
be like. You're going to have a cursor at a line and a column, and
you're going to be able to insert or delete any character where your
cursor is. This is the same kind of interface as a word editor; it is
comfortable and familiar.


### Disadvantages of Text

There's a downside, though. Since your editor represents documents as text, when
you _do_ have features like syntax highlighting or code completion, they must be
hacked together with regexps and hope. But [regexps aren't
parsers](https://stackoverflow.com/questions/1732348/regex-match-open-tags-except-xhtml-self-contained-tags#1732454),
and hope quickly leaves when you're writing emacs regexps to match perl regexps
and you're not sure how [all the backslashes got there but you think they're
multiplying](https://github.com/jrockway/cperl-mode/blob/master/cperl-mode.el#L8224).

The point I'm trying to make is that if syntax highlighting, code
completion, jump-to-definition, etc. are slow or buggy, it's probably
because they have to deal with the editor representing the document as
text. I feel confident making this assertion because these problems
become easy if you have a fully structured representation of the
document.

But maybe you don't care about that. Writing these tools is a hurdle,
but lots of software that we use everyday was a nightmare to write and
maintain. Regardless of [what it looks like underneath](FILL), it's
the interface that matters, right?

I'm tired of textual interfaces. I'm tired of writing backslashes.
`\n` is **not a newline**. It's two characters that get magically
interpreted as a newline, and if you want your string to actually
contain those two characters, you need to write three characters
instead. Why can't a string contain a newline? Some languages allow
this, but now the 

The thing is, in a tree editor, this is _trivial_. There aren't even
any questions that come up.

I'm tired of formatting wars. I don't care how code that I work on is
formatted, as long as it's consistent. Others agree, and now we have
tools like `gofmt` and `rustfmt` and dozens of others [CHECK]. But if
you're using `gofmt`, then there is a period of time in which you are
editing your code, and it might not be in standard style.

Why not go one step further? In a tree editor, there are no formatting
choices to begin with. You can't choose whether to put a curly brace
on the same line or on its own line, _because you never type a curly
brace_.

I'm tired of making syntax errors. I type a regex, and don't know
whether to escape a character or not, or I write code and make a silly
mistake. That thing I entered was a not a regex; it was not code. It
was nonsense. Why does my editor let me enter it? It's not like it's
_completely_ unaware of the structure of the language; it has syntax
highlighting after all.

In a tree editor, you cannot make a syntax error. There is no sequence
of commands you can type to ever get something syntactically invalid.
It's not like I can't deal with syntax errors; it's just that I'd
rather have my full attention elsewhere. If I'm devoting 5% of my
attention to closing delimeters, let's make that 0% instead.

<!-- I'm tired of searching for a variable name, and getting results that
are in a comment. Those are completely different kinds of things; it
doesn't make semantic sense to mix them up. -->

<!-- and hope quickly leaves after you
somehow get Paredit into a state where your lisp has an odd number of
parens, and now Paredit is dutifully enforcing that your parens never
again be balanced. -->

## Enter Synless

[FILL]

[1] I stole this phrase. Here's the original: [https://www.facebook.com/notes/kent-beck/prune-a-code-editor-that-is-not-a-text-editor/1012061842160013](Prune: A code editor that is not a text editor).

------

# Thesis

> I propose to design and construct an editor that (i) represents
> documents as an AST rather than as text, and (ii) has declarative
> knowledge of the well-formedness structure of the AST (such as which
> nodes can be children of which other nodes). This has a number of
> advantages over text editors:
>
> - **Multiple Views:** Since the document is represented as an AST,
> it can be viewed in a variety of ways, and the user can choose the
> most appropriate view for them.
>
> - **Discoverability:** Since the editor has exhaustive knowledge of
> valid document structure, it can share this knowledge with the user
> to aid discoverability in its interface.
>
> - **Safety:** Since the editor has exhaustive knowledge of valid
> document structure, it can prevent the construction of an
> ill-formed document.
>
> The thesis of the Synless project is that these advantages can
> be---but have not yet been---used to solve a number of concrete
> problems:
>
> - **Accessibility:** For editors, the fundamental accessibility
> issue is that no single way of viewing and interacting with a
> document is appropriate for all people. Synless aims to solve this
> problem by allowing a great variety of different ways to "view" the
> same document, allowing customization of the syntax, column width,
> color (or lack thereof), and modality (visual vs. screen reader).
> This is possible because a tree editor inherently separates the
> content of a document from its presentation.
>
> - **Configuration Files:** The Safety and Discoverability properties
> are well-suited to help with editing configuration files. These
> files tend to be rarely modified, so that the user may not know the
> correct formatting or which options are valid. Synless will be able
> to ensure that: (i) all possible options are presented on screen
> when editing, partly removing the need to look up documentation
> online when editing a configuration file, and (ii) only valid
> modifications may be made (at least insofar as the validity can be
> captured by a tree grammar).
>
> - **Embedded Languages:** A single document should be able to
> mix multiple languages, such as html, javascript, and css. I suspect
> this problem becomes much easier when the document is stored as an
> AST. [TODO: I think. Cite and find out.] (This is helped by Structure.)

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
