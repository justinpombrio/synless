# Synless is not a Text Editor

    I think that I shall never see
    A poem lovely as a tree.
    - Joyce Kilmer

**This whole project is in a pre-alpha stage. Even the design
  documents are under construction at best. Synless does not yet
  exist.**

## The Status Quo

Code is normally saved as, and edited as, text. Why is that?

Text is a _wildly_ successful format. If you look at any domain in
computing you'll see text everywhere:

- Data representation? CSV, JSON, and XML are all textual.
- The internet? Html, urls, and email.
- Operating systems? The Unix Philosophy proclaims: "Write
programs to handle text streams, because that is a universal interface
(Doug McIlroy)."

And editors tend to treat documents as text:

1. They present a _textual editing interface_: they have a cursor with
   a line and column, and you can insert or delete a character at any
   position.
2. They _internally represent_ documents as text (or something
   functionally equivalent to text such as a gap buffer or rope).
3. They _save and load_ documents as text.

Editing documents as text has a number of advantages. Essentially all
of them stem from the fact that it is, as Doug McIlroy says, a
"universal interface".

When a document is represented as text, you can edit it even if your
editor has no idea what kind of document it is. Sure, you won't get
any features like syntax highlighting or code completion or
jump-to-defintion. But at least you can _open_ it and make a
few changes.

Further, you know exactly what the editing interface is going to be
like. You're going to have a cursor at a line and a column, and you're
going to be able to insert or delete any character where your cursor
is. This is the same kind of interface as a word editor; it is
comfortable and familiar.


## Synless is a Tree Editor

Despite all of this, Synless is not a text editor. Instead, it is a
tree editor (a.k.a. structure editor, a.k.a. projectional editor).
That is to say:

1. Synless does not present a textual editing interface. Instead, it
   presents a tree-based interface. Its cursor is located at a node in
   the document. For example, if you resize the document (shrink the
   maximum line width), then the document will be reformatted to fit
   within that width, changing the line and column of all of its
   parts---but the cursor will still be at the same node. Along the
   same lines, you are not editing text, you are editing the tree. For
   example, you cannot type an `x` in the middle of the `if` in an
   `if` statement: the cursor can select the `if` statement, but it
   cannot select the `i` or the `f`. There is no such position.
2. Synless does not internally represent the document as text. It
   represents it only as a tree.
3. Synless _can_, however, save and load documents as text. (Though it does
   so begrudgingly.)

Considering all of the advantages of text, why might a tree editor still be
superior?

### Easier Features and Plugins

When an editor represents documents as text, features such as syntax
highlighting, automatic indentation, and code completion become a pain
to implement. They invariably rely on the structure of the code, but
all that's available is the text. Typically, they'll be hacked
together with regexes. This, however, can
[quickly get out of hand](https://github.com/jrockway/cperl-mode/blob/master/cperl-mode.el#L8230).
And you should always remember that
[regexps aren't parsers](https://stackoverflow.com/questions/1732348/regex-match-open-tags-except-xhtml-self-contained-tags#1732454).

There are of course better approaches than regexes, and editor modes
and plugins go to admirable lengths to make things work. But they can
only go so far because their job is literally impossible: what are you
supposed to do with a document with an odd number of quotation marks
(`"`) in it?

In a tree editor, by contrast, all of these features become easy, as
tools are given the code as a tree to start with. And there's no such
thing as code with an odd number of quotation marks: the tree doesn't
even contain quotation marks, it contains string literals.

### No Weird Encoding Details

Python has four kinds of string literals: they may be surrounded by
single (`'`), double (`"`), or triple (`'''` or `"""`) quotes. The
only difference between these literals is their escape behavior. For
example, single quoted strings can contain double quotes, and
vice-versa. Furthermore, triple-quoted strings can contain newlines,
but single- and double-quoted strings cannot. If you want one of these
forbidden characters, you have to "escape" it by preceding it with a
backslash.

Synless makes all of this irrelevant. String literals can contain
quote marks and newlines. Why shouldn't they be able to?

(If you think I'm making a mountain out of a molehill, I'll point out that Swift
went even further than this by adding _custom_ string delimiters, and was proud
enough to write a
[blog post](https://ericasadun.com/2018/12/26/swift-5-gives-us-nice-things-custom-string-delimiters/)
about them.)

### More Efficient Editing

I predict that Synless will be more efficient than a text editor for
most common edits. That is, most of the time if you want to write code
or edit code, you will need fewer keystrokes in Synless than you would
in, say, Vim. For writing code, this shouldn't be surprising: compare
typing out "`function(){}`" in a text editor to "`if`" for "insert
function" in Synless. For editing, it's less obvious that a tree
editor would be more efficient, because it's potentially possible that
the most efficient way to make an edit involves going through a
syntactically invalid intermediate state, which Synless obviously
can't do. So part of my prediction is that this isn't often the case.

**A Question for the Audience (yes, that's you):** I'd like to test
this, rather than just claiming it without evidence. Do you know
anywhere I could get a set of program edits as a benchmark? If you do,
would you be so kind as to open a github issue for it?

<!--
### Config Files

One way I expect Synless to be helpful is for defining and editing of
specialized configuration files. One of the advantages of a tree
editor is that you can't make syntax mistakes: there's literally no
way to enter invalid syntax. Most of the time you're working in a
language you're familiar with, and I'm sure you'd never forget a
semicolon. I certainly never do (**cough**). But configuration files
by nature all have their own specialized syntax, and it's annoying to
have to look up what you can write and how you must write it. Synless
would make the syntax discoverable. -->


## Beyond Syntax

I've seen far too many programming discussions get stuck on syntax.
It's visible and very easy to
[bikeshed](https://en.wikipedia.org/wiki/Law_of_triviality). This
isn't just an issue for novice programmers: even the Haskell committee
felt it necessary to appoint a
[syntax czar](http://haskell.cs.yale.edu/wp-content/uploads/2011/02/history.pdf)
to avoid syntactic bikeshedding. I won't go so far as to say that
syntax doesn't matter _at all_, but I'd rather have a good language
with a bad syntax than a bad language with a good syntax.

(To prove my point, I'm a fan of XSLT. If you've seen this
monstrosity, you'll know I'm serious. It's _incredibly_ verbose and
clunky. But it allows separating the content of the page from its
styling in a way that you couldn't dream of with just html and css, so
I think it's worth it.)

I want to move the discussion away from syntax and toward semantics.
The lisp community has one strategy for this: make the syntax so
boring that there's nothing _to_ talk about except semantics. Synless
takes a different approach. People are going to talk about syntax
regardless... they'll complain about the parentheses, if that's all
they see. But the syntax of a language has so little to do with the
language. Synless proves it, by letting you choose your own syntax.
You can save your choice in a style file. It's on par with a color
scheme.

That's why it's called Synless. It proves that the program you're
editing is not its syntax, by letting you view it in whatever syntax
you want.
