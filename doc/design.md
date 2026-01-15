# Design of Synless

**This is a planned architecture document. Most of it is unimplemented, and the
parts that are implemented don't necessarily match this document. Keeping this
around until it's replaced by a proper architecture document.**

Synless will be built on a client-server architecture. Let's look at
the design from the inside-out. Inside the Server is the Engine. The
Engine maintains the AST [LINK], and accepts as input a very basic set
of Commands. Encompassing the Engine is the Server. The Server
maintains the wider state of the editor, dealing with things like the
keymap. Outside of the server is a Frontend. The Frontend is
responsible for displaying things on the screen, gathering user input,
and generally being the intermediary between the user and the Server.
Finally, off to the side are some Plugins, which also talk to the
server. As a diagram:

       +--------+
       |Frontend|
       +--------+
           ^
           |
           |
           v
    +--------------+
    |              |        +------+
    |    Sever     | <----> |Plugin|
    |              |        +------+
    |  +--------+  |
    |  | Engine |  |        +------+
    |  +--------+  | <----> |Plugin|
    |              |        +------+
    +--------------+

Let's look at each of these parts in more detail.

### Engine

Every time you interact with the document in any way: by editing,
navigating, undo-ing or redo-ing, that interaction eventually gets
broken down (by the Server) into a sequence of Commands that get sent
to the Engine. The Engine is responsible for knowing how to execute
these Commands, _and for knowing how to undo that execution_. In
particular, Commands are always sent in "undo groups", and there is an
`undo` command that undoes the last "undo group".

The state of the Engine includes:

- The Document itself,
- A Cursor into the document,
- An infinite set of registers, including a 'clipboard' register for
  cutting and pasting.

Some examples of Commands include:

    (navigation) up, down, left, right
    (editing)    insert, delete
    (registers)  cut, paste, set-reg, get-reg
    (undo-ing)   undo, redo

### Server

The Frontend only deals with UI-related things, like key presses,
while the Engine only deals with very primitive Commands. The server
must bridge the gap between the two. It does so with a simple
stack-based language (similar to Postscript). The primitives of this
language are called Operations. The Frontend only needs to work with a
couple of these Operations directly: there are `keypress` and
`mousepress` Operations for user input, and a `render` Operation that
renders the document (within the screen) and returns a list of
elements to display.

When a `keypress` Operation comes in, it is looked up in the Server's
keymap file (which is saved as part of your settings, and may be
modified there). This maps the key press to a sequence of Operations,
which will do a variety of things but will generally boil down to some
Commands getting sent to the Engine to do something with the Document.

The state of the Server includes:

- The current settings, including a Keymap that maps keyboard shortcuts
- The Language that the document is written in
- Information about the screen the document will be rendered on, like
  its size in characters
- The state of the stack-based language, which (like Postscript)
  consists of:
    - A data stack
    - A call stack
    - A dictionary

Some examples of Operations include:

    (stack stuff) dup, swap, pop, dip, exec
    (dictionary)  <word>, <word> = <definition>
    (bookmarks)   ??
    (input)       keypress, mousepress
    (output)      render

### Frontend

While there is only one Engine and one Server, there need to be many
Frontends: one for each platform that Synless can run on. The first
Frontend will be for the terminal, and there may be additional ones
for the browser and for different operating systems.

A Frontend is very thin: it is responsible for forwarding keyboard and
mouse events to the Server, for rendering things onto the screen, and
for saving and loading settings and documents. Basically, everything
that might be platform-specific is the purview of the Frontend, and
everything else is the purview of the Server.

### Plugins

Like the Frontend, a Plugin interacts with the Server by sending
Operations and receiving responses. This allows them to do many
things, including navigating and editing the document, remapping keys,
mucking with the clipboard and bookmarks, and viewing and manipulating
the document directly as a tree.


## Document Representation

The document is a tree, but how should that tree be represented? It
will be commonly accessed in at least three ways, each of which
suggests a different representation:

1. The document will be processed by simple recursive descent. For
   example, laying out and rendering the document will (probably) be
   accomplished by recursive descent. This suggests a simple tree data
   structure, in which every node contains a list of its children.
2. As you edit the document, you have a cursor into it that needs to
   be able to mutate the document. This suggests a couple possible
   representations. First, the document could be represented as a
   tree, and your mutable cursor could consist of a mutable reference
   to this tree together with a path to the cursor location. Second,
   the mutable cursor could be represented as a
   [zipper](https://en.wikipedia.org/wiki/Zipper_\(data_structure\)).
3. Some nodes in the document will be remembered, for one purpose or
   another, and should be easy to recall, even if they have since
   moved. For example, you may have bookmarked a position, or have
   multiple selections, or have performed a search with multiple
   results. And in the meantime, you can edit the document, which may
   _move_ these remembered nodes. It is important that they remain
   valid even though they have moved. This suggests a representation
   of the document in which every node has a unique id, and the
   document is a hashmap from id to information about the node: its
   type, its parent (none for the root), and its list of children.

Use case (3) doesn't play well with representation (1) or
(either version of) representation (2). Thus, since we need
_something_ like the hashmap approach to handle use case (3), and
since the hashmap should work well enough for (1) and (2) as all of
the relevant operations are linear time, I choose representation (3).
Thus:

>    A Document is a map from NodeId to Node.

>    A NodeId is a unique id.

>    A Node has: a node type, an optional parent, and a list of children.

There are a couple invariants to maintain:

- `A` is the child of `B` iff `B` is the parent of `A`.
- Every node in the map is the descendant of exactly one of: (i) both
  the cursor and the document root, or (ii) a tree stored in a
  register, such as the cut stack.
