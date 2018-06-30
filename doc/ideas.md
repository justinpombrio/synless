If Paths are too big to store, they can be more compact.
    lookup x on t:
      if x == 0:
        return t
      else:
        x := x - 1
        let n = t.num_children()
        let (child, x) = (x%n, x/n)
        lookup x on t[child]
