use pretty::{Color, CursorVis, DocLabel, DocPosSpec, PaneNotation, PaneSize, Style};

pub fn make_example_pane_notation() -> PaneNotation {
    let active = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Show,
        scroll_strategy: DocPosSpec::CursorHeight { fraction: 0.6 },
    };

    let key_hints_name = PaneNotation::Doc {
        label: DocLabel::KeymapName,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let key_hints = PaneNotation::Doc {
        label: DocLabel::KeyHints,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let messages = PaneNotation::Doc {
        label: DocLabel::Messages,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let divider = PaneNotation::Fill {
        ch: '=',
        style: Style::color(Color::Base03),
    };

    let status_bar = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Proportional(1), divider.clone()),
            (PaneSize::Proportional(1), key_hints_name),
            (PaneSize::Proportional(1), divider.clone()),
        ],
    };

    PaneNotation::Vert {
        panes: vec![
            (PaneSize::Proportional(1), active),
            (PaneSize::Fixed(1), status_bar),
            (PaneSize::DynHeight, key_hints),
            (PaneSize::Fixed(1), divider),
            (PaneSize::Fixed(5), messages),
        ],
    }
}
