use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

use editor::{make_json_lang, make_singleton_lang_set, AstRef, TestEditor, TreeCmd, TreeNavCmd};
use pretty::{
    CursorVisibility, PlainText, PrettyDocument, PrettyWindow, RenderOptions, ScrollStrategy,
    WidthStrategy,
};

pub fn make_long_list(length: usize, ed: &mut TestEditor) {
    ed.exec(TreeNavCmd::Child(0)).unwrap();

    ed.exec(TreeCmd::Replace(ed.node(&"list".into()).unwrap()))
        .unwrap();
    ed.exec(TreeCmd::InsertHolePrepend).unwrap();
    ed.exec(TreeCmd::Replace(ed.node(&"true".into()).unwrap()))
        .unwrap();

    for _ in 0..(length - 1) {
        ed.exec(TreeCmd::InsertHoleAfter).unwrap();
        ed.exec(TreeCmd::Replace(ed.node(&"false".into()).unwrap()))
            .unwrap();
    }
}

pub fn render(ast_ref: AstRef) {
    let mut window = PlainText::new_infinite_scroll(80);
    let options = RenderOptions {
        scroll_strategy: ScrollStrategy::Beginning,
        cursor_visibility: CursorVisibility::Hide,
        width_strategy: WidthStrategy::Fixed(window.pane().unwrap().rect().width()),
    };

    ast_ref
        .pretty_print(&mut window.pane().unwrap(), options)
        .unwrap()
}

pub fn pretty_print(c: &mut Criterion) {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);

    let mut group = c.benchmark_group("render_lists");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    for length in &[4, 6, 8, 10, 12, 14, 16] {
        let mut ed = TestEditor::new(&lang_set, &note_set, lang_name.clone());
        make_long_list(*length, &mut ed);
        group.bench_with_input(BenchmarkId::new("list_length", *length), length, |b, _i| {
            b.iter(|| render(ed.doc.ast_ref()))
        });
    }

    group.finish()
}

criterion_group!(benches, pretty_print);
criterion_main!(benches);
