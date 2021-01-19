# Synless is not a Text Editor

    I think that I shall never see
    A poem lovely as a tree.
    - Joyce Kilmer

**This whole project is in a pre-alpha stage. Even the design documents are under construction at
best. Synless does not yet exist.**

## The Status Quo

Code is normally saved as, and edited as, text. Why is that?

Text is a _wildly_ successful format. If you look at any domain in computing you'll see text
everywhere:

- Data representation? CSV, JSON, and XML are all textual.
- The internet? Html, urls, and email.
- Operating systems? The Unix Philosophy proclaims: "Write programs to handle text streams, because
  that is a universal interface (Doug McIlroy)."

And editors tend to treat documents as text:

1. They present a _textual editing interface_: they have a cursor with a line and column, and you
can insert or delete a character at any position.
2. They _internally represent_ documents as text (or something functionally equivalent to text such
as a gap buffer or rope).
3. They _save and load_ documents as text.

Editing documents as text has a number of advantages. Essentially all of them stem from the fact
that it is, as Doug McIlroy says, a "universal interface".

When a document is represented as text, you can edit it even if your editor has no idea what kind of
document it is. Sure, you won't get any features like syntax highlighting or code completion or
jump-to-defintion. But at least you can _open_ it and make a few changes.

Further, you know exactly what the editing interface is going to be like. You're going to have a
cursor at a line and a column, and you're going to be able to insert or delete any character where
your cursor is. This is the same kind of interface as a word editor; it is comfortable and familiar.


## Synless is a Tree Editor

Despite all of this, Synless is not a text editor. Instead, it is a tree editor (a.k.a. structure
editor, a.k.a. projectional editor).  That is to say:

1. Synless does not present a textual editing interface. Instead, it presents a tree-based
interface. Its cursor is located at a node in the document. For example, if you resize the document
(shrink the maximum line width), then the document will be reformatted to fit within that width,
changing the line and column of all of its parts---but the cursor will still be at the same node.
Along the same lines, you are not editing text, you are editing the tree. For example, you cannot
type an `x` in the middle of the `if` in an `if` statement: the cursor can select the `if`
statement, but it cannot select the `i` or the `f`. There is no such position.
2. Synless does not internally represent the document as text. It represents it only as a tree.
3. Synless _can_, however, save and load documents as text. (Though it does so begrudgingly.)

Considering all of the advantages of text, why might a tree editor still be superior?

### Higher-Level Editing Commands

In Synless, if you want to delete an element in a list, you move your cursor to the element and hit
"x". This works because Synless knows the structure of your program: it knows you're cursor is on an
element of a list.

In Vim, what you must do depends on a few factors:

- If the element is short you type "d2w", meaning "delete two words", to delete the element and the
  following comma.
- Unless, that is, it's the _last_ element of the list. In that case, you have to type "bd2w".
  "d2w" wouldn't do at all: it would delete the close brakcet "]"!
- On the other hand, if the element is _long_, you need to manually select it to delete it. If it's
  all on one line and doesn't contain any nested commas (it probably doesn't), you can use "df,x"
  meaning "delete until the next comma, then delete the extra space".
- Finally, if it's long but not all on one line, you type "v" (to begin selecting), then navigate to
  its end, then "d" to delete.

Whenever you want to delete a list element, you need to figure out which of these four situations
you're in! These four cases are treated differently because to Vim, your program is a set of words
and matched brackets. (I'm not picking on Vim here. I'm using Vim as an example because it is such a
good text editor.)

**A Question for you, the Audience:**
I *suspect* that tree editing is more efficient than text editing. That is, a Vim-like structure
editor would require fewer key strokes than Vim, when performing common edits. But I can't test this
hypothesis without data, and I can't gather data without a representative sample set of program
edits as a benchmark.  Do you know of any such set?  If you do, would you be so kind as to open a
github issue for it?

### No Weird Encoding Details

Python has four kinds of string literals: they may be surrounded by single (`'`), double (`"`), or
triple (`'''` or `"""`) quotes. The only difference between these literals is their escape behavior.
For example, single quoted strings can contain double quotes, and vice-versa. Furthermore,
triple-quoted strings can contain newlines, but single- and double-quoted strings cannot. If you
want one of these forbidden characters, you have to "escape" it by preceding it with a backslash.

Synless will make all of this irrelevant. String literals can contain quote marks and newlines. Why
shouldn't they be able to?

(If you think I'm making a mountain out of a molehill, I'll point out that Swift went even further
than this by adding _custom_ string delimiters, and was proud enough to write a [blog
post](https://ericasadun.com/2018/12/26/swift-5-gives-us-nice-things-custom-string-delimiters/)
about them.)

### Easier Features and Plugins

When an editor represents documents as text, features such as syntax highlighting, automatic
indentation, and code completion become a pain to implement. They invariably rely on the structure
of the code, but all that's available is the text. Often, they're hacked together with
regexes. This, however, can [quickly get out of
hand](https://github.com/jrockway/cperl-mode/blob/master/cperl-mode.el#L8230).  And you should
always remember that [regexps aren't
parsers](https://stackoverflow.com/questions/1732348/regex-match-open-tags-except-xhtml-self-contained-tags#1732454).

There are of course better approaches than regexes, and editor modes and plugins go to admirable
lengths to make things work. But they will always face an uphill battle because their job is
_technically_ impossible: what are they supposed to do with a document with an odd number of
quotation marks (`"`) in it?

In a tree editor, by contrast, all of these features become easy, as tools are given the code as a
tree to start with. And there's no such thing as code with an odd number of quotation marks: the
tree doesn't even contain quotation marks, it contains string literals.

### Structured Documents

Developers, as a species, design an endless stream of bespoke varieties of files. Think `.gitignore`
files, or one of a company's obscure config files. If you tell Synless how to parse these documents,
it will give you not only syntax highlighting for free, but an editing interface that can never
create an invalid document. For an obscure, bespoke format, this is a pretty big deal. You won't
have to look up the obscure syntax you need; the editor will list the possibilities. This is more
up-front work than required for (say) syntax highlighting in a text editor, but with a bigger
payoff.

And if you surrender the need to have a textual syntax, you can avoid writing a parser entirely. You
tell Synless how the documents are structured, and how to print them to the screen, and you get an
editing interface customized to that language.

### Beyond Syntax

I've seen far too many programming discussions get stuck on syntax.  It's visible and very easy to
[bikeshed](https://en.wikipedia.org/wiki/Law_of_triviality). This isn't just an issue for novice
programmers: even the Haskell committee felt it necessary to appoint a [syntax
czar](http://haskell.cs.yale.edu/wp-content/uploads/2011/02/history.pdf) to avoid syntactic
bikeshedding. I won't go so far as to say that syntax doesn't matter _at all_, but I'd rather have a
good language with a bad syntax than a bad language with a good syntax.

(To prove my point, I'm a fan of XSLT. If you've seen this monstrosity, you'll know I'm serious.
It's _incredibly_ verbose and clunky. But it allows separating the content of the page from its
styling in a way that you couldn't dream of with just html and css, so I think it's worth it.)

I want to move the discussion away from syntax and toward semantics.  The lisp community has one
strategy for this: make the syntax so boring that there's nothing _to_ talk about except semantics.
Synless takes a different approach. People are going to talk about syntax regardless... they'll
complain about the parentheses, if that's all they see. But the syntax of a language has so little
to do with the language. Synless proves it, by letting you choose your own syntax.  You can save
your choice in a style file. It's on par with a color scheme.

That's why it's called Synless. It proves that the program you're editing is not its syntax, by
letting you view it in whatever syntax you want.
