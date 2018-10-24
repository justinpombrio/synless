use geometry::*;
use tree::{Path, TreeRef, match_end_of_path};
use syntax::LayoutRegion;
use syntax::Layout::*;


// Step 1: figure out where the cursor is in doc coords

pub(in render) struct CursorLocator {
    cursor_path: Path
}

// Err means "found"; Ok means "not yet found"
type Res = Result<(), Region>;

impl CursorLocator {
    pub(in render) fn locate(doc: TreeRef, doc_region: Region) -> Region {
        let mut locator = CursorLocator{
            cursor_path: doc.path().clone()
        };
        match locator.loc_tree(doc.root(), doc_region) {
            // Err means "found"; Ok means "not yet found"
            Ok(())      => err_cursor_not_found(),
            Err(region) => region
        }
    }

    fn loc_tree(&mut self, tree: TreeRef, region: Region) -> Res {
        if tree.path() == &self.cursor_path {
            Err(tree.lay_out(region).region)
        } else if self.cursor_path.starts_with(tree.path()) {
            let layout = tree.lay_out(region);
            self.loc_layout(tree, layout)
        } else {
            Ok(())
        }
    }

    fn loc_layout(&mut self, tree: TreeRef, layout: LayoutRegion) -> Res {
        match layout.layout {
            Literal(_, _) => Ok(()),
            Text(_) =>
                match match_end_of_path(&self.cursor_path, tree.path()) {
                    Some(i) => {
                        let pos = layout.region.beginning()
                            + Pos{ row: 0, col: i as Col };
                        Err(Region::char_region(pos))
                    }
                    None => Ok(())
                }
            Flush(box layout) =>
                self.loc_layout(tree, layout),
            Child(index) =>
                self.loc_tree(tree.child(index), layout.region),
            Concat(box layout1, box layout2) => {
                self.loc_layout(tree.clone(), layout1)?;
                self.loc_layout(tree, layout2)
            }
        }
    }
}

fn err_cursor_not_found() -> ! {
    panic!("render: Selection not found!")
}

#[cfg(test)]
mod tests {
    use super::*;
    use language::Language;
    use language::make_example_tree;
    
    #[test]
    fn test_locate_cursor() {
        let lang = Language::example_language();
        let doc = make_example_tree(&lang, true);

        let width = 17;
        // The rendering:
"func foo(abc,
         def) {
  'abcdef'
  + 'abc'
}";

        let region = Region{
            pos:   Pos::zero(),
            bound: Bound::infinite_scroll(width)
        };
        let tree = doc.as_ref().child(2).child(0);
        let expected = Region{
            pos: Pos{ row: 2, col: 2 },
            bound: Bound{ height: 0, width: 8, indent: 8 }
        };
        assert_eq!(CursorLocator::locate(tree, region), expected);

        let tree = doc.as_ref().child(2);
        let expected = Region{
            pos: Pos{ row: 2, col: 2 },
            bound: Bound{ height: 1, width: 8, indent: 7 }
        };
        assert_eq!(CursorLocator::locate(tree, region), expected);

        let tree = doc.as_ref();
        let expected = Region{
            pos: Pos{ row: 0, col: 0 },
            bound: Bound{ height: 4, width: 15, indent: 1 }
        };
        assert_eq!(CursorLocator::locate(tree, region), expected);

        let tree = doc.as_ref().child(1);
        let expected = Region{
            pos: Pos{ row: 0, col: 9 },
            bound: Bound{ height: 1, width: 4, indent: 3 }
        };
        assert_eq!(CursorLocator::locate(tree, region), expected);
    }
}
