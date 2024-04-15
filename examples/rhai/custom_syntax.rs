use rhai::{Dynamic, Engine};

pub fn main() {
    let mut engine = Engine::new();

    let shared = 173;

    engine
        .register_custom_syntax(["$"], false, move |_, _| Ok(Dynamic::from(shared)))
        .unwrap();

    engine.run("print($)").unwrap();
}
