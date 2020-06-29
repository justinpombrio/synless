use crate::engine::DocLabel;
use pretty::{
    Color, CursorVisibility, PaneNotation, PaneSize, RenderOptions, ScrollStrategy, Style,
    WidthStrategy,
};

pub fn make_example_pane_notation() -> PaneNotation<DocLabel> {
    let active = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        render_options: RenderOptions {
            cursor_visibility: CursorVisibility::Show,
            scroll_strategy: ScrollStrategy::CursorHeight { fraction: 0.6 },
            width_strategy: WidthStrategy::Full,
        },
    };

    let key_hints_name = PaneNotation::Doc {
        label: DocLabel::KeymapName,
        render_options: RenderOptions {
            cursor_visibility: CursorVisibility::Hide,
            scroll_strategy: ScrollStrategy::Beginning,
            width_strategy: WidthStrategy::Full,
        },
    };

    let key_hints = PaneNotation::Doc {
        label: DocLabel::KeyHints,
        render_options: RenderOptions {
            cursor_visibility: CursorVisibility::Hide,
            scroll_strategy: ScrollStrategy::Beginning,
            width_strategy: WidthStrategy::Full,
        },
    };

    let messages = PaneNotation::Doc {
        label: DocLabel::Messages,
        render_options: RenderOptions {
            cursor_visibility: CursorVisibility::Hide,
            scroll_strategy: ScrollStrategy::Beginning,
            width_strategy: WidthStrategy::Full,
        },
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
