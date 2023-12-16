# Line Numbers

In some sense, line numbers aren't meaningful in Synless:

- For screen readers, there is literally no such thing as a line.
- The notation you display with may be very different from the notation printed. E.g. you could have
  Rust displayed with indentation instead of braces, and Python displayed with braces instead of
  indentation.

However, they are important when interfacing with the outside world. Thus we'll store them exactly
when doing so! Upon save, mark every node in the document with the _first_ line number it was
printed on. Display the line number for the current node in the status bar. You can jump to a node
by line.
