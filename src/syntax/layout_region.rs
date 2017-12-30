use coord::*;
use style::Style;
use syntax::layout::BasicLayout;
use syntax::layout::BasicLayout::*;


impl BasicLayout {
    pub(crate) fn regionize(self, region: Region) -> LayoutRegion {
        regionize(self, region.bound, region.pos)
    }
}

/// A concrete plan for how to lay out the `Syntax`, once the program
/// and screen width are known.  For example, unlike in `Syntax`,
/// there is no Choice, because the choices have been resolved.
/// The outermost region always has position zero, but inner regions
/// are relative to this.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutRegion {
    pub layout: Layout,
    pub region: Region
}

/// The enum for a LayoutRegion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Display a literal string with the given style.
    Literal(String, Style),
    /// Display a text node's text with the given style.
    Text(Style),
    /// Display the layout, then a newline.
    Flush(Box<LayoutRegion>),
    /// Display the concatenation of the two layouts.
    /// The `Col` is the indent on the Bound of the first Layout.
    /// (This is redundant information, but convenient to have around.)
    Concat(Box<LayoutRegion>, Box<LayoutRegion>),
    /// Display a child node. Its Bound must be supplied.
    Child(usize)
}

fn regionize(layout: BasicLayout, bound: Bound, pos: Pos)
                               -> LayoutRegion
{
    match layout {
        Literal(s, style) => {
            LayoutRegion{
                region: Region{
                    pos: pos,
                    bound: Bound::literal(&s)
                },
                layout: Layout::Literal(s, style)
            }
        },
        Text(style, bound) => {
            LayoutRegion{
                layout: Layout::Text(style),
                region: Region{
                    pos: pos,
                    bound: bound
                }
            }
        },
        Flush(box layout) => {
            let layed = regionize(layout, bound, pos);
            LayoutRegion{
                region: Region{
                    pos:   layed.region.pos,
                    bound: layed.region.bound.flush()
                },
                layout: Layout::Flush(Box::new(layed))
            }
        },
        Child(i, bound) => {
            LayoutRegion{
                layout: Layout::Child(i),
                region: Region{
                    pos: pos,
                    bound: bound
                }
            }
        },
        Concat(box layout1, box layout2) => {
            let layed1 = regionize(layout1, bound, pos);
            let bound2 = bound.subbound_from(layed1.region.delta());
            let layed2 = regionize(layout2, bound2, layed1.region.end());
            let delta2 = layed2.region.end() - layed1.region.beginning();
            LayoutRegion{
                layout: Layout::Concat(Box::new(layed1), Box::new(layed2)),
                region: Region{
                    pos: pos,
                    bound: bound.subbound_to(delta2)
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use style::Style;
    use language::Language;
    use language::make_example_tree;

    #[test]
    fn test_regionize() {
        let small_region = Region{
            pos: Pos{ row: 2, col: 3 },
            bound: Bound{ width: 5, height: 2, indent: 2 }
        };
        let layout = Literal("ok".to_string(), Style::plain());
        assert_eq!(layout.regionize(small_region), LayoutRegion{
            layout: Layout::Literal("ok".to_string(), Style::plain()),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 2, height: 0, indent: 2 }
            }
        });
        
        let layout = Text(Style::plain(),
                          Bound{ width: 2, height: 1, indent: 3 });
        assert_eq!(layout.regionize(small_region), LayoutRegion{
            layout: Layout::Text(Style::plain()),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 2, height: 1, indent: 3 }
            }
        });

        let layout = Flush(Box::new(Text(
            Style::plain(),
            Bound{ width: 2, height: 1, indent: 3 })));
        assert_eq!(layout.regionize(small_region), LayoutRegion{
            layout: Layout::Flush(Box::new(LayoutRegion{
                layout: Layout::Text(Style::plain()),
                region: Region{
                    pos: Pos{ row: 2, col: 3 },
                    bound: Bound{ width: 2, height: 1, indent: 3 }
                }
            })),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 2, height: 2, indent: 0 }
            }
        });

        let left_layout = Child(0, Bound{ width: 3, height: 1, indent: 1 });
        let right_layout = Literal("ok".to_string(), Style::plain());
        let layout = Concat(Box::new(left_layout), Box::new(right_layout));
        let left_expected = LayoutRegion{
            layout: Layout::Child(0),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 3, height: 1, indent: 1 }
            }
        }; // 2:3-3:4;3
        let right_expected = LayoutRegion{
            layout: Layout::Literal("ok".to_string(), Style::plain()),
            region: Region{
                pos: Pos{ row: 3, col: 4 },
                bound: Bound{ width: 2, height: 0, indent: 2 }
            }
        }; // 3:4-3:6;2
        let expected = LayoutRegion{
            layout: Layout::Concat(Box::new(left_expected), Box::new(right_expected)),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 5, height: 1, indent: 3 }
            }
        }; // 2:3-3:6;5
        assert_eq!(layout.regionize(small_region), expected);
        
        let lang = Language::example_language();
        let doc = make_example_tree(&lang, false);
        let region = Region{
            pos: Pos{ row: 2, col: 3 },
            bound: Bound::infinite_scroll(20)
        };
        let lay = doc.as_ref().lay_out(region);
        // layout:
"func foo(abc, def) {
  'abcdef'
  + 'abcdef'
}";

        fn left_of_concat(lay: LayoutRegion) -> LayoutRegion {
            match lay.layout {
                Layout::Concat(box lay, _) => lay,
                _ => panic!("Expected concat")
            }
        }
        fn right_of_concat(lay: LayoutRegion) -> LayoutRegion {
            match lay.layout {
                Layout::Concat(_, box lay) => lay,
                _ => panic!("Expected concat")
            }
        }

        println!("{:?}", lay);
        assert_eq!(&format!("{}", lay.region),
                   "2:3-5:4;20");
        let lay = right_of_concat(left_of_concat(lay));
        assert_eq!(&format!("{}", lay.region),
                   "3:3-5:3;20");
    }
}
