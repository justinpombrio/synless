// Fun idea: implement dense Paths:
//   lookup x on t:
//     if x = 0 {
//       t
//     }
//     else {
//       let x = x - 1;
//       let n = t.num_children();
//       let (child, n) = (x % n, x / n);
//       lookup x on t[child]
// (For more efficiency, align to power of 2.)

/// A position in the document.
pub type Path = Vec<usize>;

/// Construct a new path that points to the `extension`'th child of
/// `path`.
pub fn extend_path(path: &Path, extension: usize) -> Path {
    let mut path = path.clone();
    path.push(extension);
    path
}

/// Construct a new path that points to the parent of `path`.
/// If `path` is to the root and has no parent, then return it
/// unchanged.
pub fn pop_path(path: &Path) -> Path {
    let mut path = path.clone();
    path.pop();
    path
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_paths() {
        let path = vec!(0, 2, 1);
        assert_eq!(extend_path(&path, 3), vec!(0, 2, 1, 3));
        assert_eq!(path, vec!(0, 2, 1));
        assert_eq!(pop_path(&path), vec!(0, 2));
        assert_eq!(path, vec!(0, 2, 1));
    }
}
