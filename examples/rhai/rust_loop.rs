use rhai::{CustomType, Engine, FnPtr, Scope, TypeBuilder, AST};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::stdin;
use std::process;
use std::rc::Rc;

#[derive(Debug, Default)]
struct Runtime {
    keymap: HashMap<char, FnPtr>,
}

impl Runtime {
    fn bind_key(&mut self, key: char, prog: FnPtr) {
        self.keymap.insert(key, prog);
    }

    fn block(&self) -> FnPtr {
        loop {
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if input.len() == 1 {
                let key = input.chars().next().unwrap();
                if let Some(ptr) = self.keymap.get(&key).cloned() {
                    return ptr;
                }
            }
            println!("Rust: unknown input");
        }
    }

    fn exit(&self) {
        process::exit(0);
    }
}

#[derive(Clone, Default)]
pub struct SharedRuntime(Rc<RefCell<Runtime>>);

impl SharedRuntime {
    fn new() -> SharedRuntime {
        SharedRuntime::default()
    }

    fn bind_key(&mut self, key: char, prog: FnPtr) {
        println!("Rust: binding key {key}");
        self.0.borrow_mut().bind_key(key, prog);
    }

    fn exit(&mut self) {
        println!("Rust: exit");
        self.0.borrow_mut().exit()
    }
}

impl CustomType for SharedRuntime {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Runtime")
            .with_fn("bind_key", SharedRuntime::bind_key)
            .with_fn("exit", SharedRuntime::exit);
    }
}

fn event_loop(engine: &mut Engine, rt: &mut SharedRuntime, ast: &AST) {
    loop {
        let prog = rt.0.borrow_mut().block();
        prog.call::<()>(&engine, &ast, ()).unwrap();
    }
}

pub fn main() {
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    let mut runtime = SharedRuntime::new();
    scope.push("r", runtime.clone());

    engine.build_type::<SharedRuntime>();

    println!("Signatures:");
    engine
        .gen_fn_signatures(false)
        .into_iter()
        .for_each(|func| println!("  {func}"));
    println!();

    let init_script = "
        let n = 1;
        r.bind_key('a', || {
            print(`a was pressed ${n} times`);
            n += 1;
        });
        r.bind_key('e', || r.exit());
    ";

    let blank_script = "";
    let blank_ast = engine.compile(blank_script).unwrap();

    let init_ast = engine.compile(init_script).unwrap();
    engine.run_ast_with_scope(&mut scope, &init_ast).unwrap();

    event_loop(&mut engine, &mut runtime, &blank_ast);
}
