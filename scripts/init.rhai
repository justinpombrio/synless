s::load_language("data/json_lang.ron");
s::open_doc("data/pokemon.json");

let keymap = new_keymap();
keymap.bind_key("h", "Say Hello", || print("hello world!"), true);
keymap.bind_key("q", "Quit", || s::abort(), true);

let layer = new_layer("default");
layer.add_mode_keymap("Tree", keymap);
s::register_layer(layer);
s::add_global_layer("default");
