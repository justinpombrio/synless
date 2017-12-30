//! Render the tree onto the screen.

mod locate_cursor;
mod position_screen;
mod render;

use coord::*;
use render::locate_cursor::CursorLocator;
use render::position_screen::position_screen;
use render::render::Renderer;

use tree::TreeRef;
use terminal::Terminal;

/// Render the tree onto the screen.
pub fn render(tree: TreeRef,
              char_index: Option<usize>,
              terminal: &mut Terminal,
              center: f32)
{
    // We will render the document onto an infinitely long scroll
    let region = Region{
        pos: Pos::zero(),
        bound: Bound::infinite_scroll(terminal.size().col)
    };

    // Step 1: figure out where the cursor is in doc coords
    let cursor_region = CursorLocator::locate(tree.clone(), region);

    // Step 2: determine where the screen should be in doc coords
    let screen_size = Bound{
        width:  terminal.size().col,
        height: terminal.size().row,
        indent: terminal.size().col
    };
    let screen_region = position_screen(screen_size, cursor_region, center);

    // Step 3: render the doc onto the screen
    Renderer::render(terminal, screen_region, tree, char_index, region);
}
