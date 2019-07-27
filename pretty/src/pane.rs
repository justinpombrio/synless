use crate::{Col, DocPosSpec, Pos, PrettyDocument, PrettyWindow, Rect, Region, Row, Shade, Style};

use std::{fmt, iter};

/// A rectangular area of a window. You can pretty-print to it, or get sub-panes
/// of it and pretty-print to those.
pub struct Pane<'a, T>
where
    T: PrettyWindow,
{
    pub(crate) window: &'a mut T,
    pub(crate) rect: Rect,
}

#[derive(Clone, Copy, Debug)]
pub enum PaneSize {
    Fixed(usize),
    Proportional(usize),
    /// There are restrictions on when you can use `DynHeight`, to keep the implementation simple:
    ///  - `DynHeight` can only be applied to subpanes within a `PaneNotation::Vert`
    ///  - a `DynHeight` subpane can only contain a `PaneNotation::Content`, not more nested subpanes
    DynHeight,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Content {
    ActiveDoc,
    ActiveDocName,
    KeyHints,
    KeymapName,
    Messages,
}

#[derive(Clone, Debug)]
pub enum PaneNotation {
    Horz {
        panes: Vec<(PaneSize, PaneNotation)>,
        style: Option<Style>,
    },
    Vert {
        panes: Vec<(PaneSize, PaneNotation)>,
        style: Option<Style>,
    },
    Content {
        content: Content,
        style: Option<Style>,
    },
    Fill {
        ch: char,
        style: Option<Style>,
    },
}

/// The visibility of the cursor in some document.
#[derive(Debug, Clone, Copy)]
pub enum CursorVis {
    Show,
    Hide,
}

#[derive(Debug)]
pub enum PaneError<T>
where
    T: fmt::Debug,
{
    NotSubPane,
    ImpossibleDemands,
    InvalidNotation,
    Missing(Content),
    /// T should be the associated `Error` type of something that implements the
    /// PrettyWindow trait.
    PrettyWindow(T),
}

impl<'a, T> Pane<'a, T>
where
    T: PrettyWindow,
{
    /// Get the position and size of the rectangular area covered by this `Pane`.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get a new `Pane` representing only the given sub-region of this `Pane`.
    /// Returns `None` if `rect` is not fully contained within this `Pane`.
    /// `rect` is specified in the same absolute coordinate system as the full
    /// `PrettyWindow` (not specified relative to this `Pane`!).
    pub fn sub_pane<'b>(&'b mut self, rect: Rect) -> Option<Pane<'b, T>> {
        if !self.rect().covers(rect) {
            return None;
        }
        Some(Pane {
            window: self.window,
            rect,
        })
    }

    /// Render a string with the given style, with its first character at the
    /// given relative position (where 0,0 is the top left corner of the
    /// `Pane`). No newlines allowed.
    pub fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.rect.pos();
        self.window.print(abs_pos, text, style)
    }

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    /// The region position is relative to the `Pane` (where 0,0 is the
    /// top left corner of the `Pane`).
    pub fn shade(&mut self, region: Region, shade: Shade) -> Result<(), T::Error> {
        let abs_region = region + self.rect.pos();
        self.window.shade(abs_region, shade)
    }

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`. The
    /// position is relative to the `Pane` (where 0,0 is the top left
    /// corner of the `Pane`).
    pub fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.rect.pos();
        self.window.highlight(abs_pos, style)
    }

    /// Render to this pane according to the given [PaneNotation], `note`. Use
    /// the `get_content` closure to map the document names used in any
    /// `PaneNotation::Content` variants to actual documents, and whether to
    /// shade that document's cursor region. If `parent_style` is not `None`,
    /// apply that style to [PaneNotation] sub-trees that don't
    /// specify their own style.
    pub fn render<F, U>(
        &mut self,
        note: &PaneNotation,
        parent_style: Option<Style>,
        get_content: F,
    ) -> Result<(), PaneError<T::Error>>
    where
        F: FnOnce(&Content) -> Option<(U, CursorVis)>,
        F: Clone,
        U: PrettyDocument,
    {
        if self.rect().is_empty() {
            // Don't try to render anything into an empty pane, just skip it.
            return Ok(());
        }

        match note {
            PaneNotation::Horz { panes, style } => {
                let child_notes: Vec<_> = panes.iter().map(|p| &p.1).collect();
                let child_sizes: Vec<_> = panes.iter().map(|p| p.0).collect();
                let total_width = usize::from(self.rect().width());
                let widths: Vec<_> = divvy(total_width, &child_sizes)
                    .ok_or(PaneError::ImpossibleDemands)?
                    .into_iter()
                    .map(|n| n as Col)
                    .collect();
                let style = style.or(parent_style);
                let rects = self.rect().horz_splits(&widths);
                for (rect, child_note) in rects.zip(child_notes.into_iter()) {
                    let mut child_pane = self.sub_pane(rect).ok_or(PaneError::NotSubPane)?;
                    child_pane.render(child_note, style, get_content.clone())?;
                }
            }
            PaneNotation::Vert { panes, style } => {
                let child_notes: Vec<_> = panes.iter().map(|p| &p.1).collect();
                let total_fixed: usize = panes.iter().filter_map(|p| p.0.get_fixed()).sum();
                let total_height = self.rect().height();
                let mut available_height = total_height.saturating_sub(total_fixed as Row);
                let child_sizes = panes
                    .iter()
                    .map(|p| match p.0 {
                        PaneSize::DynHeight => {
                            // Convert dynamic height into a fixed height, based on the currrent document.
                            if let PaneNotation::Content { content, .. } = &p.1 {
                                let f = get_content.clone();
                                let (doc, _) =
                                    f(content).ok_or(PaneError::Missing(content.to_owned()))?;
                                let height =
                                    available_height.min(doc.required_height(self.rect().width()));
                                available_height -= height;
                                Ok(PaneSize::Fixed(height as usize))
                            } else {
                                // DynHeight is only implemented for Content subpanes!
                                Err(PaneError::InvalidNotation)
                            }
                        }
                        size => Ok(size), // pass through all other pane sizes
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let heights: Vec<_> = divvy(total_height as usize, &child_sizes)
                    .ok_or(PaneError::ImpossibleDemands)?
                    .into_iter()
                    .map(|n| n as Row)
                    .collect();
                let style = style.or(parent_style);

                let rects = self.rect().vert_splits(&heights);
                for (rect, child_note) in rects.zip(child_notes.into_iter()) {
                    let mut child_pane = self.sub_pane(rect).ok_or(PaneError::NotSubPane)?;
                    child_pane.render(child_note, style, get_content.clone())?;
                }
            }
            PaneNotation::Content { content, style } => {
                // TODO how to use style? pretty_print doesn't take it.
                let _style = style.or(parent_style).unwrap_or_default();
                let width = self.rect().width();

                let (doc, cursor_visibility) =
                    get_content(content).ok_or(PaneError::Missing(content.to_owned()))?;
                doc.pretty_print(width, self, DocPosSpec::CursorAtTop, cursor_visibility)?;
            }
            PaneNotation::Fill { ch, style } => {
                let style = style.or(parent_style).unwrap_or_default();
                let line: String = iter::repeat(ch)
                    .take(self.rect().width() as usize)
                    .collect();
                let rows = self.rect().height();
                for row in 0..rows {
                    self.print(Pos { row, col: 0 }, &line, style)?;
                }
            }
        }
        Ok(())
    }
}

impl PaneSize {
    fn get_fixed(&self) -> Option<usize> {
        match self {
            PaneSize::Fixed(n) => Some(*n),
            _ => None,
        }
    }

    fn get_proportional(&self) -> Option<usize> {
        match self {
            PaneSize::Proportional(n) => Some(*n),
            _ => None,
        }
    }
}

impl<T> From<T> for PaneError<T>
where
    T: fmt::Debug,
{
    fn from(e: T) -> PaneError<T> {
        PaneError::PrettyWindow(e)
    }
}

fn divvy(cookies: usize, demands: &[PaneSize]) -> Option<Vec<usize>> {
    let total_fixed: usize = demands.iter().filter_map(|demand| demand.get_fixed()).sum();
    if total_fixed > cookies {
        return None; // Impossible to satisfy the demands!
    }

    let hungers: Vec<_> = demands
        .iter()
        .filter_map(|demand| demand.get_proportional())
        .collect();

    let mut proportional_allocation =
        proportionally_divide(cookies - total_fixed, &hungers).into_iter();

    Some(
        demands
            .iter()
            .map(|demand| match demand {
                PaneSize::Fixed(n) => *n,
                PaneSize::Proportional(_) => proportional_allocation.next().expect("bug in divvy"),
                PaneSize::DynHeight => {
                    panic!("All DynHeight sizes should have been replaced by Fixed sizes by now!")
                }
            })
            .collect(),
    )
}

/// Divvy `cookies` up among children as fairly as possible, where the `i`th
/// child has `child_hungers[i]` hunger. Children should receive cookies in proportion
/// to their hunger, with the difficulty that cookies cannot be split into
/// pieces. Exact ties go to the leftmost tied child.
fn proportionally_divide(cookies: usize, child_hungers: &[usize]) -> Vec<usize> {
    let total_hunger: usize = child_hungers.iter().sum();
    // Start by allocating each child a guaranteed minimum number of cookies,
    // found as the floor of the real number of cookies they deserve.
    let mut cookie_allocation: Vec<usize> = child_hungers
        .iter()
        .map(|hunger| cookies * hunger / total_hunger)
        .collect();
    // Compute the number of cookies still remaining.
    let allocated_cookies: usize = cookie_allocation.iter().sum();
    let leftover: usize = cookies - allocated_cookies;
    // Determine what fraction of a cookie each child still deserves, found as
    // the remainder of the above division. Then hand out the remaining cookies
    // to the children with the largest remainders.
    let mut remainders: Vec<(usize, usize)> = child_hungers
        .iter()
        .map(|hunger| cookies * hunger % total_hunger)
        .enumerate()
        .collect();
    remainders.sort_by(|(_, r1), (_, r2)| r2.cmp(r1));
    remainders
        .into_iter()
        .take(leftover)
        .for_each(|(i, _)| cookie_allocation[i] += 1);
    // Return the maximally-fair cookie allocation.
    cookie_allocation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proportional_division() {
        assert_eq!(proportionally_divide(0, &vec!(1, 1)), vec!(0, 0));
        assert_eq!(proportionally_divide(1, &vec!(1, 1)), vec!(1, 0));
        assert_eq!(proportionally_divide(2, &vec!(1, 1)), vec!(1, 1));
        assert_eq!(proportionally_divide(3, &vec!(1, 1)), vec!(2, 1));
        assert_eq!(proportionally_divide(4, &vec!(10, 11, 12)), vec!(1, 1, 2));
        assert_eq!(proportionally_divide(5, &vec!(17)), vec!(5));
        assert_eq!(proportionally_divide(5, &vec!(12, 10, 11)), vec!(2, 1, 2));
        assert_eq!(proportionally_divide(5, &vec!(10, 10, 11)), vec!(2, 1, 2));
        assert_eq!(proportionally_divide(5, &vec!(2, 0, 1)), vec!(3, 0, 2));
        assert_eq!(proportionally_divide(61, &vec!(1, 2, 3)), vec!(10, 20, 31));
        assert_eq!(
            proportionally_divide(34583, &vec!(55, 98, 55, 7, 12, 200)),
            vec!(4455, 7937, 4454, 567, 972, 16198)
        );
    }

}
