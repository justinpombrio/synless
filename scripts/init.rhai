// TODO split into user_init.rhai and system_init.rhai

fn make_candidate_keymap() {
    let keymap = new_keymap();
    keymap.bind_key("esc", "ExitMenu", || s::escape());
    keymap.bind_key("up", "Up", || s::menu_selection_up(), false);
    keymap.bind_key("down", "Down", || s::menu_selection_down(), false);
    keymap.bind_key("bksp", "Backspace", || s::menu_selection_backspace(), false);
    keymap
}

fn make_candidate_node_selection_keymap(language_name) {
    let keymap = make_candidate_keymap();
    for construct in s::language_constructs(s::get_language(language_name)) {
        keymap.add_regular_candidate(s::construct_name(construct), construct);
    }
    keymap.bind_key_for_regular_candidates("enter", "Select", |construct| construct);
    keymap
}

fn make_char_node_selection_keymap(language_name) {
    let keymap = new_keymap();
    for construct in s::language_constructs(s::get_language(language_name)) {
        let key = s::construct_key(construct);
        if key != "" {
            let construct_copy = construct;
            let prog = || construct_copy;
            keymap.bind_key(key, s::construct_name(construct), prog);
        }
    }
    keymap.bind_key("esc", "ExitMenu", || s::escape());
    keymap
}

fn open_file_menu(dir) {
    let dir = s::canonicalize_path(dir);
    let contents = s::list_files_and_dirs(dir);
    let keymap = new_keymap();
    for file in contents.files {
        keymap.add_regular_candidate(s::path_file_name(file), file);
    }
    for child_dir in contents.dirs {
        let child_dir_copy = child_dir;
        let prog =  || open_file_menu(child_dir_copy);
        keymap.bind_key_for_special_candidate(
            "enter",
            s::path_file_name(child_dir_copy) + "/",
            "ViewDir",
            prog
        );
    }
    keymap.bind_key_for_special_candidate("enter", "..", "ViewDir", || {
        open_file_menu(dir + "/..");
    });

    s::open_menu("file_selection", `Open file in ${dir}`, keymap);
}

s::load_language("data/keyhints_lang.ron");
s::load_language("data/selection_lang.ron");
s::load_language("data/json_lang.ron");
s::load_language("data/string_lang.ron");

s::open_doc("data/pokemon.json");

// ~~~ Tree Keymap ~~~

let tree_keymap = new_keymap();
tree_keymap.bind_key("q", "Quit", || s::abort());
tree_keymap.bind_key("o", "Open", || {
    open_file_menu(s::current_dir());
});

tree_keymap.bind_key("k", "Prev", || s::tree_nav_prev());
tree_keymap.bind_key("K", "First", || s::tree_nav_first());
tree_keymap.bind_key("j", "Next", || s::tree_nav_next());
tree_keymap.bind_key("J", "Last", || s::tree_nav_last());
tree_keymap.bind_key("l", "FirstChild", || s::tree_nav_first_child());
tree_keymap.bind_key("L", "LastChild", || s::tree_nav_last_child());
tree_keymap.bind_key("h", "Parent", || s::tree_nav_parent());
tree_keymap.bind_key("tab", "NextLeaf", || s::tree_nav_next_leaf());
tree_keymap.bind_key("S-tab", "PrevLeaf", || s::tree_nav_prev_leaf());

tree_keymap.bind_key(";", "Parent", || s::tree_nav_parent());
tree_keymap.bind_key("^", "First", || s::tree_nav_first());
tree_keymap.bind_key("$", "Last", || s::tree_nav_last());

tree_keymap.bind_key("enter", "EnterText", || s::tree_nav_enter_text());
tree_keymap.bind_key("bksp", "Backspace", || s::tree_ed_backspace());
tree_keymap.bind_key("del", "Delete", || s::tree_ed_delete());
tree_keymap.bind_key("u", "Undo", || s::undo());
tree_keymap.bind_key("r", "Redo", || s::redo());

tree_keymap.bind_key("m", "SaveBookmark", || s::save_bookmark('a'));
tree_keymap.bind_key("'", "GoToBookmark", || s::goto_bookmark('a'));

tree_keymap.bind_key("y", "Copy", || s::copy());
tree_keymap.bind_key("d", "Cut", || s::cut());
tree_keymap.bind_key("p", "Paste", || s::paste());
tree_keymap.bind_key("P", "PasteSwap", || s::paste_swap());

tree_keymap.bind_key("I", "Insert", || {
    s::open_menu("candidate_node_selection", "Select node to insert");
    let construct = s::block();
    s::insert_node(construct);
});

tree_keymap.bind_key("i", "QuickInsert", || {
    s::open_menu("char_node_selection", "Select node to insert");
    let construct = s::block();
    s::insert_node(construct);
});

// ~~~ Text Keymap ~~~

let text_keymap = new_keymap();
text_keymap.bind_key("esc", "ExitText", || s::text_nav_exit());
text_keymap.bind_key("enter", "ExitText", || s::text_nav_exit());
text_keymap.bind_key("left", "Left", || s::text_nav_left());
text_keymap.bind_key("right", "Right", || s::text_nav_right());
text_keymap.bind_key("bksp", "Backspace", || s::text_ed_backspace());
text_keymap.bind_key("del", "Delete", || s::text_ed_delete());
text_keymap.bind_key("tab", "NextLeaf", || {
    s::text_nav_exit();
    s::tree_nav_next_text();
    s::tree_nav_enter_text();
});
text_keymap.bind_key("S-tab", "PrevLeaf", || {
    s::text_nav_exit();
    s::tree_nav_prev_text();
    s::tree_nav_enter_text();
});

// ~~~ File Selection Keymap ~~~

let file_selection_keymap = make_candidate_keymap();
file_selection_keymap.bind_key_for_regular_candidates("enter", "OpenFile", |path| s::open_doc(path));

// ~~~ Default Layer ~~~

let layer = new_layer("default");

layer.add_mode_keymap("Tree", tree_keymap);
layer.add_mode_keymap("Text", text_keymap);

layer.add_menu_keymap("file_selection", file_selection_keymap);
// TODO: use local layers
layer.add_menu_keymap("candidate_node_selection", make_candidate_node_selection_keymap("json"));
layer.add_menu_keymap("char_node_selection", make_char_node_selection_keymap("json"));

s::register_layer(layer);
s::add_global_layer("default");
