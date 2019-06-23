use crate::{Col, Pane, Pos, PrettyDocument, PrettyWindow, Row, Style};
use std::{fmt, iter};

#[derive(Clone, Copy)]
pub enum PaneSize {
    Fixed(usize),
    Proportional(usize),
}

#[derive(Clone)]
pub enum Content {
    ActiveDoc,
    KeyHints,
}

#[derive(Clone)]
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

#[derive(Debug)]
pub enum Error<T>
where
    T: fmt::Debug,
{
    NotSubPane,
    ImpossibleDemands,
    Content,
    /// T should be the associated `Error` type of something that implements the
    /// PrettyWindow trait.
    PrettyWindow(T),
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

impl<T> From<T> for Error<T>
where
    T: fmt::Debug,
{
    fn from(e: T) -> Error<T> {
        Error::PrettyWindow(e)
    }
}

pub fn render_pane<T, F, U>(
    pane: &mut Pane<T>,
    note: &PaneNotation,
    parent_style: Option<Style>,
    get_content: F,
) -> Result<(), Error<T::Error>>
where
    T: PrettyWindow,
    F: FnOnce(&Content) -> Option<U>,
    F: Clone,
    U: PrettyDocument,
{
    match note {
        PaneNotation::Horz { panes, style } => {
            let child_notes: Vec<_> = panes.iter().map(|p| &p.1).collect();
            let child_sizes: Vec<_> = panes.iter().map(|p| p.0).collect();
            let total_width = usize::from(pane.rect().width());
            let widths: Vec<_> = divvy(total_width, &child_sizes)
                .ok_or(Error::ImpossibleDemands)?
                .into_iter()
                .map(|n| n as Col)
                .collect();
            let style = style.or(parent_style);
            for (rect, child_note) in pane
                .rect()
                .horz_splits(&widths)
                .zip(child_notes.into_iter())
            {
                let mut child_pane = pane.sub_pane(rect).ok_or(Error::NotSubPane)?;
                render_pane(&mut child_pane, child_note, style, get_content.clone())?;
            }
        }
        PaneNotation::Vert { panes, style } => {
            let child_notes: Vec<_> = panes.iter().map(|p| &p.1).collect();
            let child_sizes: Vec<_> = panes.iter().map(|p| p.0).collect();
            let total_height = pane.rect().height() as usize;
            let heights: Vec<_> = divvy(total_height, &child_sizes)
                .ok_or(Error::ImpossibleDemands)?
                .into_iter()
                .map(|n| n as Row)
                .collect();
            let style = style.or(parent_style);
            for (rect, child_note) in pane
                .rect()
                .vert_splits(&heights)
                .zip(child_notes.into_iter())
            {
                let mut child_pane = pane.sub_pane(rect).ok_or(Error::NotSubPane)?;
                render_pane(&mut child_pane, child_note, style, get_content.clone())?;
            }
        }
        PaneNotation::Content { content, style } => {
            // TODO how to use style?
            let _style = style.or(parent_style).unwrap_or_default();
            let width = pane.rect().width();
            let doc = get_content(content).ok_or(Error::Content)?;

            // Put the top of the cursor at the top of the pane.
            // TODO support fancier positioning options.
            let cursor_region = doc.locate_cursor(width);
            let doc_pos = Pos {
                col: 0,
                row: cursor_region.pos.row,
            };
            doc.pretty_print(width, pane, doc_pos)?;
        }
        PaneNotation::Fill { ch, style } => {
            let style = style.or(parent_style).unwrap_or_default();
            let line: String = iter::repeat(ch)
                .take(pane.rect().width() as usize)
                .collect();
            let rows = pane.rect().height();
            for row in 0..rows {
                pane.print(Pos { row, col: 0 }, &line, style)?;
            }
        }
    }
    Ok(())
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
