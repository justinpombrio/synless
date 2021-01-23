# Synless Walkthrough

**This whole project is in a pre-alpha stage. Even the design
  documents are under construction at best. Synless does not yet
  exist.**

What will it be like to work with Synless, once it's done? I'll try to describe the experience.

(To be extra clear, this is hypothetical. Ain't none of this exists yet 'cept plans.)


## Adding Support for a bit of Rust

What would it take to add support for Rust to Synless... but just enough to write a "hello world"
program?

    // Hello world, in Rust
    fn main() {
        println!("Hello world!")
    }

We like to think of "hello world" as simple, but this short program actually has six kinds of nodes
in it. Let's go through them one at a time. By the end you'll have a good sense of what it's like to
add support for a language to Synless.

### Function Nodes

Synless needs a bunch of information about each kind of node:

- What to call it in help text
- A one-letter shortcut key for editing
- Where the node is allowed to appear (called its _sort_)
- What children it has
- How to display it

Here is all of that, for Rust function nodes:

    node function {
        name: "function definition"
        key: f
        sort: Statement
        children {
            name: Identifier
            params: ParameterList
            body: Block
        }
        notation:
              fn $name($params) { $body }
            | fn $name($params) {
                  $body
              }
            | fn $name(
                  $params
              ) {
                  $body
              }
    }

Let's go through the pieces.

**Name** is just a very short description. It will get shown in some help dialogs, when editing.

**Key** is a one letter key used to refer to this kind of node when editing. For example, `i` means
"insert new node", and expects to be followed by one of these keys. So we're saying that `i` then
`f` should insert a new function.

**Sort** says where this kind of node is allowed to appear. For example, it distinguishes between
Expressions and Statements.

**Children** lists the node's children. A function has three children: the name of the
function, its parameters, and its body. Each of the children has a _sort_ listed after the ':'.
This matches the `sort` field we just saw, and one node is allowed to be the child of another when
these two sorts agree.

(I'm simplifying here! Functions in Rust contain a lot more than just these three children. They
can also have type parameters, and a visibility modifier, and other things as well. I'm ignoring
them for expediency.)

**Notation** says how to display this node to the screen. In this case, there are three possible
layouts (separated by `|`s) depending on whether you need a newline before `$body`, and whether
you need a newline before `$params`.

Synless tries to keep the lines shorter than your preferred width (famously 80 characters, but of
course it's customizable). It will try each of the three layouts in order, and pick the first
layout whose _first line_ fits in the width.

It would be better for Synless to pick the first layout such that _all_ of its lines fit in the
preferred width. This would let you specify fancier notations, and have Synless pick between them
intelligently. However, we tried that and it was _way_ too slow. From
[our experiments](https://github.com/justinpombrio/partial-pretty-printer/tree/master/tests), the
first line rule is _reasonably_ expressive, and blindingly fast.

A notation also contains style information, like whether a word like `fn` in it should be bolded or
colored. I'm ignoring that here.

### Identifier Nodes

We declared above that a function name is an Identifier. But we don't have any nodes of sort
`Identifier` yet. Let's add one:

    node identifier {
        name: "Identifier"
        key: i
        sort: Identifier
        children: text /[a-zA-Z][a-zA-Z0-9_]*|_[a-zA-Z0-9_]+/
        notation: $text
    }

An identifier doesn't really have children; it just contains text. Nodes like this are called
_texty_. In this declaration, a texty node says `text` for its children, and gives a Regex that the
text must obey.

Its notation can then use `$text` to refer to its text contents. Identifiers should be displayed as-is,
with no additional trappings, so `$text` is the entire notation.

### Block Nodes

The body of a function is a list of statements separated by semicolons. This is called a _block_:

    node block {
        name: "block of statements"
        key: none
        sort: Block
        children: list Statement
        notation:
            list {
                zero: 
                one: $elem
                many:
                    $elem;
                    $rest
            }
    }

This node has yet another kind of children: it contains a list of zero or more nodes of sort
Statement. Nodes like this are called _listy_. So altogether, there are three kinds of nodes:

- **Fixed**, when there are a fixed number of children, that have (potentially) different sorts.
- **Listy**, when there is a list of children, all of the same sort.
- **Texty**, where the node just contains text.

How a listy node should be displayed depends on how many elements it has, and its notation reflects
that. Here, we're saying that:

- `zero`: if there are no statements, don't print anything;
- `one`: if there is exactly one statement, print it; and
- `many`: if there are two or more statements, separate them by semicolons and newlines.

### String, Macro, and Args Nodes

Those nodes have introduced all the important concepts. If you're curious, here are the rest of the
node types we'll need to write "hello world". And if you're not curious, you can skip this section.

    node string {
        name: "string constant"
        key: s
        sort: Expression,
        children: text /.*/
        notation: "$text"
    }

    node simple_macro_call {
        name: "macro call"
        key: m
        sort: Expression,
        children {
            macro: Identifier
            args: ArgList
        }
        notation:
              $macro($args)
            | $macro(
                  $args
              )
    }

    node args {
        name: "argument list"
        sort: ArgList
        children: list Expression
        notation:
            list {
                zero:
                one: $elem
                many:
                      $elem, $rest
                    | $elem,
                      $rest
            }
    }

There's a wrinkle around macro calls. Technically, a macro call can contain an arbitrary token tree,
and can appear in a variety of places in the program (not just as an expression, but also at the top
level). I'm not sure what the best way to handle all of this is. For simplicity, we'll force macro
arguments to be expressions.

## Writing Hello World in Rust

With those basic node types, we've given Synless enough knowledge of Rust that we can use it to
write a "Hello world" program.

The program starts as a simple empty hole:

    ?

_Insert a function with `if`._ `i` for "insert", and `f` for "function" because we declared function
nodes to have `key: f` above.

    fn ?() {}

The cursor is on the function.

_Edit the function name with `l [Enter]`._ `l` means "go to first child", which is the function name.
Enter means "edit text".

_Name the function by typing `main [Enter]`._ Enter means "finish text".

    fn main() {}

_Insert a statement into the function body with `jjo`._ `j` means "go to right sibling", and `o`
means "insert a hole into the list". 

    fn main() {
        ?
    }

The cursor is now on the new hole.

_Insert a macro call with `im`._

    fn main() {
        ?!()
    }

_Give the macro name with `l [Enter] println [Enter]`._

    fn main() {
        println!()
    }

_Give it an argument with `jo`._

    fn main() {
        println!(?)
    }

_Insert a string literal with `is [Enter] Hello world! [Enter]`._

    fn main() {
        println!("Hello world!")
    }

There's the program!

It took 38 key strokes, which is about the same as typing it out in a text editor.  (The program
text is 40 characters total; you might be able to enter it with fewer key strokes depending on the
text editor.)

## Searching

What does searching look like in Synless?

You might expect that if a string of characters appears on the screen (or _would_ appear on the
screen if you scrolled to a different part of the program), then you could search for this string
and find it. But that's not how search works! There are two reasons it doesn't work like this:

- Synless does not store the text of the whole document.
- While Synless won't have screen reader support any time soon, it will one day. And on that day,
  the navigation and editing commands with a screen reader will be exactly the same as those
  without. Since the way a document is _rendered_ may be quite different with a screen reader, the
  search functionality cannot depend on the notation.

Instead, you search for a program fragment. For example, let's say you forgot where you said "Hello"
in the program:

_Type `/` to search._ Your search fragment is now a hole:

    ?

_Type `is` to insert a string literal, since you want to search in strings._ 

    "?"

The thing you have inserted is _similar to_ a Rust string literal, but modified for searching.
Instead of containing text, it contains a _text search_. There are a few different node types that
you can put here, depending on whether you want to search for a perfect match, or for a substring,
or search by regex.

_Type `is` to insert a substring search._ Previously `s` meant "Rust string literal". Now it means
"substring search". This happened because the letter that comes after `i` is context sensitive: it
depends on the _sort_ of the hole you are replacing.

    "[substr]?"

_Type `l [Enter] Hello [Enter]` to fill in the substring to search for._

    "[substr]Hello"

_Type `/` to actually do the search._

This will search _only_ for string literals containing "Hello". Variables, types, and keywords named
"Hello" won't get caught up.

## Large Documents

We've put a lot of effort into making Synless fast for large documents. At the heart of it is a
pretty printing algorithm that can print _just what's on the screen_, and nothing more.

(Ok, that's an approximation. It prints as little else as it can. But in practice it can skip almost
all of a large document, besides what's on the screen.)

As a result, Synless can perform two operations very quicky:

1. Resize a document, by changing the preferred maximum line width.
2. Jump from one part of the document to another.

Neither of these operations take time proportional to the size of the document. Instead, Synless
always re-renders the screen from scratch. [TODO: Let's see how practical this is! Preliminary
performance test was very encouraging.]

This pretty printing algorithm is implemented in the [partial pretty printing
crate](https://github.com/justinpombrio/partial-pretty-printer).
