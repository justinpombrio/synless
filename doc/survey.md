# Synless

Synless will be a general purpose
[structure editor][wiki:structure_editor].
Or projectional editor. Or tree editor.
We have too many names for this thing.
How about we call it an AST editor instead?

This page is my partial categorization of tree editors. I made it
before discovering the
[Reddit list of projectional editors](https://www.reddit.com/r/nosyntax/wiki/projects),
however, which is a much more complete list. You should probably go
there instead.

### Collaborative Editors
Here's a [collaborative rich text editor](https://ckeditor.com/blog/Lessons-learned-from-creating-a-rich-text-editor-with-real-time-collaboration/).

## Motivation

Here are some other people's motivation for writing a tree editor:

- [An Experiment in Structured Code Editing - Isomorf](https://blog.isomorf.io/an-experiment-in-structured-code-editing-68b917a9157c)
  ([lobste.rs](https://lobste.rs/s/ofx5mf/experiment_structured_code_editing))
  This feature list matches what I want to build.
- [Prune: A Code Editor that is Not a Text Editor](https://www.facebook.com/notes/kent-beck/prune-a-code-editor-that-is-not-a-text-editor/1012061842160013/)
- [Why Don't We Have a General Purpose Tree Editor?](http://pcmonk.me/2014/04/01/why-dont-we-have-a-general-purpose-tree-editor.html)
([HN](https://news.ycombinator.com/item?id=13578256))
- [Code as text is a problem](http://dohgramming.com/post/code-as-text-is-a-problem/)
  ([HN](https://news.ycombinator.com/item?id=14278605))
- [Language Oriented Programming: The Next Programming Paradigm](http://www.onboard.jetbrains.com/is1/articles/04/10/lop/)
- A [Hacker News comment](https://news.ycombinator.com/item?id=14675431)
  about passing structured data on the command line.
- More [dicussion](https://news.ycombinator.com/item?id=13773813) on Hacker News, with many links.


## Survey

See also the [Reddit list of projectional editors](https://www.reddit.com/r/nosyntax/wiki/projects).

### Audacious Projects to Reinvent all of Programming

- [Subtext: uncovering the simplicity of programming][subtext].
- [Unison: next-generation programming platform][unison].
- [Lava][lava] Ok, maybe this one just wants to reinvent imperitive
  class-based languages with inheritance.

### Language Workbenches

- [Spoofax][spoofax]
- [Jetbrains MPS][mps]


### Hip Tree Editors

- [Isomorf][isomorf] with online demo ([HN][hn:isomorph]). First
  impressions: great discoverability (entered a switch statement
  without knowing its syntax); annoying to scroll with arrow keys
  (inorder navigation); surprisingly difficult to enter numbers; my
  fibonacci function failed with an "unevaluatable" error; slow
  (sometimes had a half-second delay after keypress, or to fill an
  autocomplete box).
- [Flense][flense] for Clojure, by Max Kreminski
- [Plastic][plastic] for ClojureScript, by Antonin Hildebrand. I'm a
  little confused here, because ClojureScript is a compiler, not a
  language?
- [Cirru][cirru]: "edit S-Expression and generate Clojure".
  Expands on s-expression syntax to allow indentation to (optionally)
  replace parentheses. Has an online demo of a (rather limited) tree
  editor.
- [Lamdu][lamdu]: tree editor and live coding environment.
- [Projectured][projectured]: a "general purpose tree editor written in
  common lisp".
- [Scheme Bricks][bricks].


### Academic Papers *about* Tree Editing

- [On the Usefulness of Syntax Directed Editors][lang] - Lang,
  INRIA. Based on experiences with Mentor.

### Venerable Old Academic Tree Editors

- [Programming Environments based on Structured Editors: the Mentor Experience][mentor] -
  Juillet, INREA 1980.
- [Cornell Program Synthesizer][teitelbaum] - Teitelbaum and Reps,
  CACM 1981.
- [Pan][pan] - Ballance et al, technical report from UC Berkley 1990.
- [The Synthesizer Generator][syngen] - Reps and Teitelbaum,
  SIGSOFT 1984. Synthesize an editor from a language specification.

### Itty Bitty Prototypes that Get It

- [Fructure in DrRacket](https://github.com/disconcision/fructure).

### Not quite Tree Editors

- [Frame-Based Editing][frame] ([HN][hn:frame]). Every line is text,
  but lines are organized into a tree of nested frames.
- [Tree Sheets][tree_sheets]: a free form data organizer, like
  spreadsheets but for trees.
- [Parinfer][parinfer]: lisp editor that can infer parentheses from
  indentation. This sounds scary, but it actually looks very well
  designed.
- [Paredit]() is a treeish
  editor glued onto emacs for editing elisp and its ilk. It does not
  balance parentheses, however.  For example, C-w does not preserve
  balance.
- [Structured Haskell Mode][haskell] for emacs. I haven't tested it
  yet, but don't give it good odds of guaranteeing well-formedness.


### Quite not Tree Editors

- Here's what looks like a good guide to
[Writing a Text Editor](https://viewsourcecode.org/snaptoken/kilo/)
[HN](https://news.ycombinator.com/item?id=14046446).
No talk of undo, though!
- [prettier][prettier]: an opinionated code formatter.


### Dead Links

- [http://www.programtree.com/](http://www.programtree.com/)
- [http://www.guilabs.net/](http://www.guilabs.net/)


[wiki:structure_editor]:https://en.wikipedia.org/wiki/Structure_editor
[teitelbaum]:http://pages.cs.wisc.edu/~fischer/papers/synthesizer.pdf
[plastic]:https://github.com/darwin/plastic
[flense]:https://github.com/mkremins/flense
[pan]:http://www.ics.uci.edu/~andre/ics228s2006/ballancegrahamvandevanter.pdf
[frame]:https://kclpure.kcl.ac.uk/portal/files/71018111/Frame_based_editing.pdf
[hn:frame]:https://news.ycombinator.com/item?id=14609215
[syngen]:https://www.ics.uci.edu/~taylor/ics228/SynGen.pdf
[lang]:http://bat8.inria.fr/~lang/papers/trondheim86/usefulness-syntax-directed-editors-19860616-18.pdf
[tree_sheets]:http://strlen.com/treesheets/
[subtext]:http://www.subtext-lang.org/
[mentor]:https://hal.inria.fr/file/index/docid/76535/filename/RR-0026.pdf
[lava]:http://lavape.sourceforge.net/index.htm
[isomorf]:https://isomorf.io/?#!/tours/~
[hn:isomorf]:https://news.ycombinator.com/item?id=15532964#15533742
[spoofax]:http://www.metaborg.org/en/latest/
[mps]:https://www.jetbrains.com/mps/concepts/
[cirru]:http://cirru.org/
[lamdu]:http://www.lamdu.org/
[unison]:http://unisonweb.org/
[paredit]:https://www.emacswiki.org/emacs/ParEdit
[haskell]:https://github.com/chrisdone/structured-haskell-mode
[projectured]:https://github.com/projectured/projectured
[parinfer]:https://github.com/shaunlebron/parinfer
[prettier]:https://github.com/prettier/prettier
[bricks]:http://www.pawfal.org/dave/index.cgi?Projects/Scheme%20Bricks

## TODO

- [Deuce](https://news.ycombinator.com/item?id=17398705)
