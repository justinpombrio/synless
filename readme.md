![](https://github.com/justinpombrio/synless/workflows/Tests/badge.svg)

# Synless

**This whole project is in a pre-alpha stage. Even the design
  documents are under construction at best. Synless does not yet
  exist.**

------

Synless is a hypothetical tree editor. It hopes to one day grow up to
be a real tree editor. It aims to:

- Provide better editing commands, that act directly on the structure of the program, rather than on
  its textual representation.
- Eliminate the need for weird encoding details like escape sequences (I'm looking at you, quadruple
  backslashes).
- Make features and plugins much easier to write, by always knowing the exact structure of the
  document. (It can do this because it never has to try to parse an incomplete and syntactically
  invalid program.)
- Make it easy to design new structured document formats, and to provide an editor for them that can
  never create an invalid document.
- End formatting wars by delegating formatting choices to the same status as style files.

Synless is not:

- A text editor.
- A tree editor built on top of a text editor. There's no
  gap buffer. It's really just a tree.
- A language workbench. Synless will not help you define a language
  semantics or perform static analysis.

To learn more:

[Why Synless? And why "Synless"?](doc/why.md)

[Synless Walkthrough](doc/walkthrough.md)

[The Synless Documentation (to come)](doc/readme.md)

[The Synless Design Documentation](doc/design.md) (for developers)

[An Incomplete Survey of Tree Editors](doc/survey.md)
