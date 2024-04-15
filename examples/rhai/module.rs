use rhai::{Engine, FuncRegistration, Module};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
struct Runtime {}

impl Runtime {
    fn method_1(&mut self) -> i64 {
        1
    }
    fn method_2(&mut self) -> i64 {
        2
    }
}

pub fn main() {
    let mut engine = Engine::new();

    let runtime = Rc::new(RefCell::new(Runtime {}));
    let mut runtime_module = Module::new();

    let cloned_runtime = runtime.clone();
    FuncRegistration::new("method_1")
        .in_internal_namespace()
        .set_into_module(&mut runtime_module, move || {
            cloned_runtime.borrow_mut().method_1()
        });

    let cloned_runtime = runtime.clone();
    FuncRegistration::new("method_2")
        .in_internal_namespace()
        .set_into_module(&mut runtime_module, move || {
            cloned_runtime.borrow_mut().method_2()
        });

    engine.register_static_module("s", runtime_module.clone().into());

    let script = "
        print(s::method_1());
        print(s::method_2());
    ";
    engine.run(script).unwrap();
}
