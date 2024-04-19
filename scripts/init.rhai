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

let keymap = new_keymap();
keymap.bind_key("h", "Say Hello", || print("hello world!"));
keymap.bind_key("q", "Quit", || s::abort());
keymap.bind_key("o", "Open", || {
    open_file_menu(s::current_dir());
});

let file_selection_keymap = new_keymap();
file_selection_keymap.bind_key_for_regular_candidates("enter", "Open file", |path| s::open_doc(path));
file_selection_keymap.bind_key("esc", "Exit menu", || s::escape());
file_selection_keymap.bind_key("up", "Up", || s::menu_selection_up(), false);
file_selection_keymap.bind_key("down", "Down", || s::menu_selection_down(), false);
file_selection_keymap.bind_key("bksp", "Backspace", || s::menu_selection_backspace(), false);

let layer = new_layer("default");
layer.add_menu_keymap("file_selection", file_selection_keymap);
layer.add_mode_keymap("Tree", keymap);
s::register_layer(layer);
s::add_global_layer("default");
