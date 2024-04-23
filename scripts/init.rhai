// TODO split into user_init.rhai and system_init.rhai

fn open_file_menu(dir) {
    let contents = s::list_files_and_dirs(dir);
    let keymap = new_keymap();
    for file in contents.files {
        keymap.add_regular_candidate(file);
    }
    for child_dir in contents.dirs {
        let prog =  || {
            // TODO could we hit recursion depth limit when opening many dirs?
            open_file_menu(child_dir);
        };
        keymap.bind_key_for_special_candidate("enter", child_dir, "View directory", prog);
    }
    keymap.bind_key_for_special_candidate("enter", "..", "View directory", || {
        open_file_menu(dir + "/..");
    });

    s::open_menu("file_selection", keymap);
}

s::load_language("data/keyhints_lang.ron");
s::load_language("data/selection_lang.ron");
s::load_language("data/json_lang.ron");

s::open_doc("data/pokemon.json");

// ~~~ Tree Keymap ~~~

let tree_keymap = new_keymap();
tree_keymap.bind_key("h", "Say Hello", || print("hello world!"));
tree_keymap.bind_key("q", "Quit", || s::abort());
tree_keymap.bind_key("o", "Open", || {
    open_file_menu(s::current_dir());
});

tree_keymap.bind_key("left", "Inorder prev", || s::tree_nav_inorder_prev());
tree_keymap.bind_key("right", "Inorder next", || s::tree_nav_inorder_next());
tree_keymap.bind_key("down", "Right sibling", || s::tree_nav_next());
tree_keymap.bind_key("up", "Left sibling", || s::tree_nav_prev());

tree_keymap.bind_key("j", "Left sibling", || s::tree_nav_prev());
tree_keymap.bind_key("k", "Right sibling", || s::tree_nav_next());
tree_keymap.bind_key("h", "Inorder next", || s::tree_nav_inorder_prev());
tree_keymap.bind_key("l", "Inorder prev", || s::tree_nav_inorder_next());
tree_keymap.bind_key("J", "First", || s::tree_nav_first());
tree_keymap.bind_key("K", "Last", || s::tree_nav_last());
tree_keymap.bind_key(";", "Parent", || s::tree_nav_parent());

tree_keymap.bind_key("^", "First", || s::tree_nav_first());
tree_keymap.bind_key("$", "Last", || s::tree_nav_last());
tree_keymap.bind_key("enter", "Enter text", || s::tree_nav_enter_text());
tree_keymap.bind_key("bksp", "Backspace", || s::tree_ed_backspace());
tree_keymap.bind_key("del", "Delete", || s::tree_ed_delete());
tree_keymap.bind_key("u", "Undo", || s::undo());
tree_keymap.bind_key("r", "Redo", || s::redo());

tree_keymap.bind_key("m", "Save bookmark", || s::save_bookmark('a'));
tree_keymap.bind_key("'", "Go to bookmark", || s::goto_bookmark('a'));

// ~~~ Text Keymap ~~~

let text_keymap = new_keymap();
text_keymap.bind_key("esc", "Exit text", || s::text_nav_exit());
text_keymap.bind_key("left", "Left", || s::text_nav_left());
text_keymap.bind_key("right", "Right", || s::text_nav_right());
text_keymap.bind_key("bksp", "Backspace", || s::text_ed_backspace());
text_keymap.bind_key("del", "Delete", || s::text_ed_delete());

// ~~~ File Selection Keymap ~~~

let file_selection_keymap = new_keymap();
file_selection_keymap.bind_key_for_regular_candidates("enter", "Open file", |path| s::open_doc(path));
file_selection_keymap.bind_key("esc", "Exit menu", || s::escape());
file_selection_keymap.bind_key("up", "Up", || s::menu_selection_up(), false);
file_selection_keymap.bind_key("down", "Down", || s::menu_selection_down(), false);
file_selection_keymap.bind_key("bksp", "Backspace", || s::menu_selection_backspace(), false);

let layer = new_layer("default");
layer.add_menu_keymap("file_selection", file_selection_keymap);
layer.add_mode_keymap("Tree", tree_keymap);
layer.add_mode_keymap("Text", text_keymap);
s::register_layer(layer);
s::add_global_layer("default");
