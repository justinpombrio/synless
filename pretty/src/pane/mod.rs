mod pane_notation;
mod proportionally_divide;

pub use pane_notation::{PaneNotation, PaneSize};

use crate::geometry::{Col, Pos, Rect, Row};
use crate::pretty_print::{
    pretty_print, CursorVisibility, PrettyDocument, PrettyWindow, ScrollStrategy,
};
use proportionally_divide::proportionally_divide;
use std::marker::PhantomData;
use std::{fmt, iter};

/// Errors that can occur while attempting to render to a `Pane`.
#[derive(Debug)]
pub enum PaneError<E, L>
where
    E: fmt::Debug,
    L: Clone,
{
    NotSubPane,
    ImpossibleDemands,
    InvalidNotation,
    MissingLabel(L),
    /// T should be the associated `Error` type of something that implements the
    /// PrettyWindow trait.
    PrettyWindow(E),
}

impl<E, L> From<E> for PaneError<E, L>
where
    E: fmt::Debug,
    L: Clone,
{
    fn from(err: E) -> PaneError<E, L> {
        PaneError::PrettyWindow(err)
    }
}

pub fn render_panes<W, D, L, F>(
    window: &mut W,
    notation: &PaneNotation<L>,
    get_content: &F,
) -> Result<(), PaneError<W::Error, L>>
where
    W: PrettyWindow,
    D: PrettyDocument,
    L: Clone,
    F: Fn(&L) -> Option<(D, CursorVisibility)>,
{
    let rect = Rect::new(Pos::zero(), window.size()?);
    Ok(Renderer::new(window, get_content).render(rect, notation)?)
}

struct Renderer<'a, W, D, L, F>
where
    W: PrettyWindow,
    D: PrettyDocument,
    L: Clone,
    F: Fn(&L) -> Option<(D, CursorVisibility)>,
{
    window: &'a mut W,
    get_content: &'a F,
    _label: PhantomData<L>,
    _document: PhantomData<D>,
}

impl<'a, W, D, L, F> Renderer<'a, W, D, L, F>
where
    W: PrettyWindow,
    D: PrettyDocument,
    L: Clone,
    F: Fn(&L) -> Option<(D, CursorVisibility)>,
{
    fn new(window: &'a mut W, get_content: &'a F) -> Renderer<'a, W, D, L, F> {
        Renderer {
            window,
            get_content,
            _label: PhantomData,
            _document: PhantomData,
        }
    }

    fn render(
        &mut self,
        rect: Rect,
        notation: &PaneNotation<L>,
    ) -> Result<(), PaneError<W::Error, L>> {
        if rect.is_empty() {
            // Don't try to render anything into an empty pane, just skip it.
            return Ok(());
        }
        match notation {
            PaneNotation::Horz { panes } => {
                let child_notations: Vec<_> = panes.iter().map(|p| &p.1).collect();
                let child_sizes: Vec<_> = panes.iter().map(|p| p.0).collect();
                let total_width = usize::from(rect.width());
                let widths: Vec<_> = divvy(total_width, &child_sizes)
                    .ok_or(PaneError::ImpossibleDemands)?
                    .into_iter()
                    .map(|n| n as Col)
                    .collect();
                let rects = rect.horz_splits(&widths);
                for (child_rect, child_notation) in rects.zip(child_notations.into_iter()) {
                    self.render(child_rect, child_notation)?;
                }
            }
            PaneNotation::Vert { panes } => {
                let child_notations: Vec<_> = panes.iter().map(|p| &p.1).collect();
                let total_height = rect.height();
                let child_sizes = resolve_dynheight(panes, rect, total_height, self.get_content)?;
                let heights: Vec<_> = divvy(total_height as usize, &child_sizes)
                    .ok_or(PaneError::ImpossibleDemands)?
                    .into_iter()
                    .map(|n| n as Row)
                    .collect();
                let rects = rect.vert_splits(&heights);
                for (child_rect, child_notation) in rects.zip(child_notations.into_iter()) {
                    self.render(child_rect, child_notation)?;
                }
            }
            PaneNotation::Doc { label } => {
                let (doc, cursor_visibility) =
                    (self.get_content)(label).ok_or(PaneError::MissingLabel(label.to_owned()))?;
                let ss = ScrollStrategy::CursorAtTop;
                pretty_print(doc, self.window, rect, ss, cursor_visibility)?;
            }
            PaneNotation::Fill { ch, style } => {
                let line: String = iter::repeat(ch).take(rect.width() as usize).collect();
                let rows = rect.height();
                for row in 0..rows {
                    let pos = rect.pos() + Pos { row, col: 0 };
                    self.window.print(pos, &line, *style)?;
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

fn resolve_dynheight<E, D, L, F>(
    panes: &[(PaneSize, PaneNotation<L>)],
    rect: Rect,
    total_height: Row,
    get_content: &F,
) -> Result<Vec<PaneSize>, PaneError<E, L>>
where
    E: fmt::Debug,
    D: PrettyDocument,
    L: Clone,
    F: Fn(&L) -> Option<(D, CursorVisibility)>,
{
    let total_fixed: usize = panes.iter().filter_map(|p| p.0.get_fixed()).sum();
    let mut available_height = total_height.saturating_sub(total_fixed as Row);
    let mut child_sizes = vec![];
    for (size, notation) in panes {
        match size {
            PaneSize::DynHeight => {
                // Convert dynamic height into a fixed height, based on the currrent document.
                if let PaneNotation::Doc { label, .. } = &notation {
                    let (doc, _) =
                        get_content(label).ok_or(PaneError::MissingLabel(label.to_owned()))?;
                    let height = available_height.min(doc.required_height(rect.width()));
                    available_height -= height;
                    child_sizes.push(PaneSize::Fixed(height as usize))
                } else {
                    // DynHeight is only implemented for Doc subpanes!
                    return Err(PaneError::InvalidNotation);
                }
            }
            _ => child_sizes.push(*size), // pass through all other pane sizes
        }
    }
    Ok(child_sizes)
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
