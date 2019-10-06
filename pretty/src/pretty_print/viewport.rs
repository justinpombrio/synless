use super::pretty_window::PrettyWindow;
use crate::geometry::{Pos, Rect, Region};
use crate::style::{Shade, Style};
use std::cmp;

/// What part of the document to show.
#[derive(Debug, Clone)]
pub enum ScrollStrategy {
    /// Put this row and column of the document at the top left corner of the Pane.
    Fixed(Pos),
    /// Put the top edge of the cursor at the top of the Pane.
    CursorAtTop,
}

/// A viewport from the window into the document. You can ask it to render
/// things _in document coordinates_, and it will know how to transform and
/// (when necessary) clip them before asking the Window to render them.
pub struct Viewport<'a, W: PrettyWindow> {
    window: &'a mut W,
    doc_pos: Pos,   // Pos<DocCoords>
    win_pos: Pos,   // Pos<WinCoords>
    doc_rect: Rect, // Rect<DocCoords>
}

impl<'a, W: PrettyWindow> Viewport<'a, W> {
    pub fn new(
        window: &'a mut W,
        scroll_strategy: &ScrollStrategy,
        cursor_region: Region,
        win_rect: Rect,
    ) -> Viewport<'a, W> {
        // Determine positions `doc_pos` on the document and `win_pos` on the
        // window that ought to be aligned.
        let doc_pos: Pos;
        let win_pos: Pos;
        match scroll_strategy {
            ScrollStrategy::Fixed(pos) => {
                doc_pos = *pos;
                win_pos = win_rect.pos();
            }
            ScrollStrategy::CursorAtTop => {
                doc_pos = cursor_region.pos;
                win_pos = win_rect.pos();
            }
        }
        // Take `win_rect` and translate it to document coordinates, clipping as
        // needed.
        // TODO: implement saturating_sub for points
        let top = (win_rect.rows.0 + doc_pos.row).saturating_sub(win_pos.row);
        let bot = (win_rect.rows.1 + doc_pos.row).saturating_sub(win_pos.row);
        let left = (win_rect.cols.0 + doc_pos.col).saturating_sub(win_pos.col);
        let right = (win_rect.cols.1 + doc_pos.col).saturating_sub(win_pos.col);
        let upper_left = Pos {
            row: top,
            col: left,
        };
        let size = Pos {
            row: bot - top,
            col: right - left,
        };
        let clipped_rect = Rect::new(upper_left, size);
        Viewport {
            window,
            doc_pos,
            win_pos,
            doc_rect: clipped_rect,
        }
    }

    /// Render a string with the given style, located at `text_region` in the
    /// document. No newlines allowed.
    pub fn print(&mut self, text: &str, text_region: Region, style: Style) -> Result<(), W::Error> {
        if !text_region.overlaps_rect(self.doc_rect) {
            return Ok(()); // Not much to show!
        }
        // Calculate the start and end _character position_.
        let start_char = self.doc_rect.pos().col.saturating_sub(text_region.pos.col);
        let end_char = cmp::min(
            text_region.width(),
            self.doc_rect.cols.1 - text_region.pos.col,
        );
        // Turn those character positions into byte indices.
        let mut chars = text.char_indices();
        let start_byte = chars
            .nth(start_char as usize)
            .expect("Viewport - issue with transform_text()")
            .0;
        let end_byte = chars
            .nth((end_char - start_char - 1) as usize)
            .map(|x| x.0)
            .unwrap_or(text.len());
        // Clip the text and show it.
        let mut win_pos = text_region.pos;
        win_pos.col -= start_char;
        let clipped_text = &text[start_byte..end_byte];
        self.window.print(win_pos, clipped_text, style)
    }

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    pub fn shade(&mut self, region: Region, shade: Shade) -> Result<(), W::Error> {
        match self.transform_region(region) {
            None => Ok(()),
            Some(region) => self.window.shade(region, shade),
        }
    }

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`.
    pub fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), W::Error> {
        match self.transform_pos(pos) {
            None => Ok(()),
            Some(pos) => self.window.highlight(pos, style),
        }
    }

    /// Check whether a document region is visible through the viewport.
    pub fn is_region_visible(&self, region: Region) -> bool {
        self.transform_region(region).is_some()
    }

    fn transform_pos(&self, pos: Pos) -> Option<Pos> {
        if self.doc_rect.contains(pos) {
            Some(pos + self.win_pos - self.doc_pos)
        } else {
            None
        }
    }

    /// Transform a document region into window coordinates, clipping as needed.
    /// Returns `None` if there's nothing left after clipping.
    fn transform_region(&self, region: Region) -> Option<Region> {
        let mut region = region.crop(self.doc_rect);
        region.pos = self.transform_pos(region.pos)?;
        Some(region)
    }
}
