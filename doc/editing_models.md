# Editing Models

## On

In the "On" model, the cursor is _on_ a node. Insertion is complicated, because
the cursor can't go all of the places you might want to insert a node, such as
after all nodes, or inside an empty list. For this reason, "adding stuff" is
split into two phases:

1. i/a/o -- insert before,after,child adds a hole
2. r/p -- replace the hole, or paste over the hole

**Advantages:** easy to display.

**Disadvantages:** _requires_ i/a/o for editing.

## Phantom

A variant of "on" where there's a "phantom node" at the end of every listy
sibling list. That way you can navigate to it to insert.

**Advantages:** easy to display.

**Disadvantages:** display is hard: either you always show it (a completely
unacceptable amount of noise) or you show it only when the list is selected
(producing "flickering").

## Between

The cursor is _between_ nodes. Insertion and deletion works like inserting and
deleting single characters in a regular text editor. Thus you only need one
basic insertion operation, and you don't need holes in listy nodes, and in fact
_never need to insert a hole_ because they only need show up in incomplete fixed
nodes.

What does down do? If it goes to the beginning of the node to the left, then
"down down" never works. On the other hand, if you just inserted a node, its now
to your left, and you may want to go to its beginning. One nice thing for "down"
to do is to go to the end of the previous node or the beginning of the next
node, both of which are adjacent to the cursor.

Display is hard. Some options:

- Highlight the previous node, drawing a bar to its right, unless you're at the
  beginning in which case you highlight the next node and draw a bar to its
  right, unless you're at an empty in which case you do something else. The
  downsides of this are its complexity, and the fact that it's a lie: your
  cursor isn't _on_ a node, it's _between_ nodes.
- Highlight the previous node in gray and the next node in black. Works nicely
  for everything but empty lists, where you perhaps insert a single black
  square, maybe with a cdot.

**Advantages:** fewer required operations, no holes needed.

**Disadvantages:** hard to display, what does down do.

## On-Or-Before

The cursor can be on a node (typical), or before the first element of a listy
node. Insertion on a fixed node replaces it with a hole; deletion on a fixed
node overwrites it; insertion on a listy node inserts a new element after;
deletion on a listy node removes the current element and shifts left.

Since the cursor can be before the first element of a listy sequence, you only
need one basic insertion operation. (A fixed node has N children, N insertion
points, and N allowed cursor locations. A listy node has N children, N+1
insertion points, and N+1 allowed cursor locations.)

Display is annoying in that you need to have some way of indicating that you're
before the first element, it needs to work even in an empty list, and it needs
to be visually distinct from a text cursor.

**Advantages:** fewer required operations, no holes needed.

**Disadvantages:** hard to visually display "you're at the start of this list,
before any of its elements".

This design is what we ultimately went with. The Between approach was clever but
proved really hard to use when actually implemented.

## Comparison of edits

    ~On~
    i/I - insert hole before/beginning
    a/A - insert hole after/end
    o/O - insert hole as first/last child
    r - replace node with construct
    d - remove node
    x - cut node
    p - paste over node
    j/J - first/last child

    ~Between~
    ^/$ - goto beginning/end of siblings
    o/O - goto beginning/end of children
    k - goto beginning of left node
    r - insert construct
    d - remove/delete
    x - cut
    p - paste before
    j/J - end of left node / beginning of right node

     On        | Between   | Operation
    -----------+-----------+---------------------
     Ap        | $p        | paste at end of list
     arlorl    | rljrl     | [[]]
     ar+jr1lr2 | r+kr1r2   | 1 + 2
     xap       | xlp       | swap
