//! Now fixed: https://github.com/rhaiscript/rhai/issues/862
//! The fix is on master, and will be released with 1.19.0

use rhai::{Dynamic, Engine};

fn main() {
    let mut engine = Engine::new();
    let shared_state = 173; // In reality, would be an Rc<RefCell<_>>

    #[allow(deprecated)]
    engine.on_var(move |name, _, _| {
        if name == "state" {
            Ok(Some(Dynamic::from(shared_state)))
        } else {
            Ok(None)
        }
    });

    engine.run("state").unwrap(); // variable found
    engine.run("fn f() { state }; f()").unwrap(); // variable found
    engine.run("|| state").unwrap(); // variable not found
}
