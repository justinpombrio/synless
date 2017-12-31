extern crate rustbox;

use std::iter::Iterator;

use coord::*;
use tree::{Path, extend_path, match_end_of_path};
use doc::{TreeRef};
use syntax::LayoutRegion;
use syntax::Layout::*;
use style::{Style, Shade};
use terminal::{Terminal};


// Step 3: render the doc onto the screen

pub(in render) struct Renderer<'a> {
    terminal:    &'a mut Terminal,
    screen:      Region,
    cursor_path: Path,
    shading:     Vec<(Region, Shade)>
}

impl<'a> Renderer<'a> {
    pub(in render) fn render(terminal: &'a mut Terminal,
                             screen: Region,
                             tree: TreeRef,
                             char_index: Option<usize>,
                             doc_region: Region) {
        let cursor_path = match char_index {
            None => tree.path().clone(),
            Some(i) => extend_path(tree.path(), i)
        };
        let mut renderer = Renderer{
            terminal:    terminal,
            screen:      screen,
            cursor_path: cursor_path,
            shading:     vec!()
        };
        renderer.render_tree(tree.root(), doc_region);
    }

    fn render_tree(&mut self, tree: TreeRef, region: Region) {
        // Lay it out
        let layout = tree.lay_out(region);
        let region = layout.region;
        // Check if there's anything to show at all
        if !region.overlaps(self.screen) {
            return;
        }
        // Shade the background
        let shade = self.shading_depth(tree.path());
        self.shade_region(region, shade);
        // Render it
        self.render_layout(tree, layout);
    }

    fn render_layout(&mut self, tree: TreeRef, layout: LayoutRegion) {
        match layout.layout {
            Literal(text, style) => {
                let pos = layout.region.beginning();
                self.render_str(text.chars(), pos, style);
            }
            Text(style) => {
                let pos = layout.region.beginning();
                self.highlight_selected_char(pos, tree.path());
                self.render_str(tree.text().chars(), pos, style);
            }
            Flush(box layout) => {
                self.render_layout(tree, layout);
            }
            Child(index) => {
                let child = tree.child(index);
                self.render_tree(child, layout.region);
            }
            Concat(box layout1, box layout2) => {
                self.render_layout(tree.clone(), layout1);
                self.render_layout(tree, layout2);
            }
        }
    }

    fn render_str<Text>(&mut self, text: Text, pos: Pos, style: Style)
        where Text : Iterator<Item = char>
    {
        for (i, ch) in text.enumerate() {
            let pos = pos + Pos{ col: i as Col, row: 0 };
            self.render_char(ch, pos, style);
        }
    }

    fn render_char(&mut self, ch: char, pos: Pos, mut style: Style) {
        let pos = self.screen.transform(pos)
            .expect("render: Text out of bounds");
        for &(ref region, ref shade) in &self.shading {
            if region.contains(pos) {
                style.shade = *shade;
            }
        }
        if style.shade == Shade(0) {
            style.emph.bold = true;
        }
        self.terminal.print_char(ch, pos, style);
    }

    fn highlight_selected_char(&mut self, pos: Pos, path: &Path) {
        match match_end_of_path(&self.cursor_path, path) {
            None => (),
            Some(i) => {
                let pos = pos + Pos{ row: 0, col: i as Col };
                let region = Region::char_region(pos);
                self.shade_region(region, Shade(0));
            }
        }
    }

    fn shading_depth(&self, path: &Path,) -> Shade {
        // Shading is len(selected) - len(common_prefix)
        let prefix_len = common_prefix_len(path, &self.cursor_path);
        Shade(self.cursor_path.len() - prefix_len)
    }

    fn shade_region(&mut self, region: Region, shade: Shade) {
        self.terminal.shade_region(region, shade);
        self.shading.push((region, shade));
    }
}
