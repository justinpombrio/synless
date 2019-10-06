use crate::style::Style;

/// Specify the size of a subpane within a vertically or horizontally concatenated set of subpanes.
#[derive(Clone, Copy, Debug)]
pub enum PaneSize {
    /// Give the subpane exactly this number of rows of height (for
    /// `PaneNotation::Vert`) or columns of width (for `PaneNotation::Horz`).
    Fixed(usize),

    /// Try to give the subpane exactly the amount of height needed to fit its
    /// content. If that's not possible, give it all of the remaining height.
    /// This means that if there are multiple DynHeight subpanes and not enough
    /// height to satisfy all of them, the ones earlier in the list get
    /// priority. `DynHeight` subpanes get priority over `Proportional`
    /// subpanes, regardless of order.
    ///
    /// There are restrictions on when you can use `DynHeight`, to keep the implementation simple:
    ///  - `DynHeight` can only be applied to subpanes within a `PaneNotation::Vert`
    ///  - a `DynHeight` subpane can only contain a `PaneNotation::Doc`, not more nested subpanes
    DynHeight,

    /// After `Fixed` and `DynHeight` subpanes have been assigned a
    /// width/height, divide up the remaining available width/height between the
    /// `Proportional` subpanes according to their given weights. The size of
    /// each subpane will be proportional to its weight, so that a subpane with
    /// weight 2 will be twice as large as one with weight 1, etc.
    Proportional(usize),
}

/// Specify the content of a `Pane`. `L` is the type of "document labels".
#[derive(Clone, Debug)]
pub enum PaneNotation<L> {
    /// Split the pane horizontally into multiple subpanes, each with its own
    /// `PaneNotation`. Each subpane has the same height as this `Pane`, and a
    /// width determined by its `PaneSize`. Optionally apply a `style` to any
    /// subpane that doesn't already specify its own style.
    Horz {
        panes: Vec<(PaneSize, PaneNotation<L>)>,
    },
    /// Split the pane vertically into multiple subpanes, each with its own
    /// `PaneNotation`. Each subpane has the same width as this `Pane`, and a
    /// height determined by its `PaneSize`. Optionally apply a `style` to any
    /// subpane that doesn't already specify its own style.
    Vert {
        panes: Vec<(PaneSize, PaneNotation<L>)>,
    },
    /// Render a `PrettyDocument` into this `Pane`. The given `DocLabel` will
    /// be used to dynamically look up a `PrettyDocument` every time the `Pane`
    /// is rendered.
    Doc { label: L },
    /// Fill the entire `Pane` by repeating the given character. Optionally
    /// apply a `style`.
    Fill { ch: char, style: Style },
}
