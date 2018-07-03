If Paths are too big to store, they can be more compact.

    lookup x on t:
      if x == 0:
        return t
      else:
        x := x - 1
        let n = t.num_children()
        let (child, x) = (x%n, x/n)
        lookup x on t[child]

Code completion:

    Should support most common two use cases:
    1. Quick use for spelling-correctness/speed
    2. Searching for method name or attribute or whatever.
    See paper by Luke Church et al.
