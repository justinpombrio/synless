//! Technically, this is a feature.

use rhai::{CustomType, Engine, TypeBuilder};

#[derive(Clone, Default)]
struct Struct {}

impl Struct {
    fn method(&mut self) -> i64 {
        3
    }
}

impl CustomType for Struct {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Struct")
            .with_fn("new_struct", Struct::default)
            .with_fn("method", Struct::method);
    }
}

fn main() {
    let mut engine = Engine::new();
    engine.build_type::<Struct>();

    let script = "
        fn method() {
            print(`Called rhai function named 'method'`);
        }

        let s = new_struct();
        s.method();
    ";

    engine.run(script).unwrap();
}
