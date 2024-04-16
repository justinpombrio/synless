use rhai::{
    CustomType, Dynamic, Engine, EvalAltResult, FnPtr, FuncRegistration, Module, Position, Scope,
    TypeBuilder,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::stdin;
use std::process;
use std::rc::Rc;

// Features:
// x escape to exit all menus
// x close_menu=true              close_menu; switch_buffer
// x close_menu=false             inc_font_size; block
// x chain menus
// x fatal errors
// x non fatal errors

#[derive(Clone)]
struct EditorError {
    is_fatal: bool,
    message: String,
}

impl CustomType for EditorError {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("EditorError")
            .with_get("is_fatal", |err: &mut EditorError| -> bool { err.is_fatal })
            .with_get("message", |err: &mut EditorError| -> String {
                err.message.clone()
            });
    }
}

impl From<EditorError> for Box<EvalAltResult> {
    fn from(editor_error: EditorError) -> Self {
        Box::new(EvalAltResult::ErrorRuntime(
            Dynamic::from(editor_error),
            Position::NONE,
        ))
    }
}

#[derive(Debug, Default)]
struct Runtime {
    keymaps: HashMap<String, HashMap<char, KeyProg>>,
    active_menu: String,
    count: u64,
}

#[derive(Debug, Clone)]
struct KeyProg {
    prog: FnPtr,
    close_menu: bool,
}

impl CustomType for KeyProg {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("KeyProg")
            .with_get_set(
                "prog",
                |kp: &mut KeyProg| -> FnPtr { kp.prog.clone() },
                |kp: &mut KeyProg, prog: FnPtr| kp.prog = prog,
            )
            .with_get_set(
                "close_menu",
                |kp: &mut KeyProg| -> bool { kp.close_menu },
                |kp: &mut KeyProg, close_menu: bool| kp.close_menu = close_menu,
            );
    }
}

impl Runtime {
    fn bind_key(&mut self, keymap: &str, key: char, close_menu: bool, prog: FnPtr) {
        let keyprog = KeyProg { prog, close_menu };
        if !self.keymaps.contains_key(keymap) {
            self.keymaps.insert(keymap.to_owned(), HashMap::new());
        }
        self.keymaps.get_mut(keymap).unwrap().insert(key, keyprog);
    }

    fn open_menu(&mut self, menu: &str) -> Result<(), Box<EvalAltResult>> {
        if !self.keymaps.contains_key(menu) {
            return Err(EditorError {
                is_fatal: true,
                message: format!("Unknown menu name: {}", menu),
            }
            .into());
        }

        self.active_menu = menu.to_owned();
        Ok(())
    }

    fn close_menu(&mut self) {
        self.active_menu = "Default".to_owned();
    }

    fn block_for_keyprog(&self) -> KeyProg {
        print!("MENU: {} (", self.active_menu);
        if let Some(keymap) = &self.keymaps.get(&self.active_menu) {
            for key in keymap.keys() {
                print!("{} ", key);
            }
        }
        println!(")");
        loop {
            let key = {
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input.len() != 1 {
                    println!("Rust: unknown input");
                    continue;
                }
                input.chars().next().unwrap()
            };
            if let Some(keymap) = self.keymaps.get(&self.active_menu) {
                if let Some(keyprog) = keymap.get(&key).cloned() {
                    return keyprog;
                }
                println!("Rust: unknown input");
            } else {
                println!("Rust: unknown menu");
            }
        }
    }

    fn exit(&self) {
        process::exit(0);
    }

    fn increment(&mut self) -> u64 {
        self.count += 1;
        self.count
    }

    fn decrement(&mut self) -> Result<u64, Box<EvalAltResult>> {
        if self.count == 0 {
            Err(EditorError {
                is_fatal: false,
                message: "decrement".to_owned(),
            }
            .into())
        } else {
            self.count -= 1;
            Ok(self.count)
        }
    }

    fn count(&self) -> u64 {
        self.count
    }
}

macro_rules! register {
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*) ) => {
        let rt = $runtime.clone();
        let closure = move | $( $param : $type ),* | {
            rt.borrow_mut().$method( $( $param ),* )
        };
        FuncRegistration::new(stringify!($method))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
}

fn register_runtime_methods(module: &mut Module) {
    let runtime = Rc::new(RefCell::new(Runtime {
        keymaps: HashMap::new(),
        active_menu: "Default".to_owned(),
        count: 0,
    }));

    register!(module, runtime.open_menu(menu: &str));
    register!(module, runtime.close_menu());
    register!(module, runtime.bind_key(keymap: &str, key: char, close_menu: bool, prog: FnPtr));
    register!(module, runtime.block_for_keyprog());
    register!(module, runtime.exit());
    register!(module, runtime.increment());
    register!(module, runtime.decrement());
    register!(module, runtime.count());
}

pub fn main() {
    let mut engine = Engine::new();
    engine.set_fail_on_invalid_map_property(true);

    engine.build_type::<KeyProg>();
    engine.build_type::<EditorError>();

    println!("Signatures:");
    engine
        .gen_fn_signatures(false)
        .into_iter()
        .for_each(|func| println!("  {func}"));
    println!();

    let prelude_script = "
        fn block() {
            loop {
                let keyprog = s::block_for_keyprog();
                if keyprog.close_menu {
                    s::close_menu();
                    return call(keyprog.prog);
                }
                call(keyprog.prog);
            }
        }
        
        fn escape() {
            throw `escape`;
        }

        fn allow_error(f) {
            try {
                call(f)
            } catch (err) {
                if !err.is_fatal {
                    let msg = err.message;
                    print(`ignoring non-fatal error: ${msg}`);
                } else {
                    throw err;
                }
            }
        }
    ";

    let init_script = "
        // Default Menu
        s::bind_key(`Default`, 'c', true, || s::open_menu(`ClosureCounter`));
        s::bind_key(`Default`, 'i', true, || {
            s::open_menu(`Node`);
            let node = block();
            // return to main loop
            print(`  Inserting node of type ${node}`);
        });
        s::bind_key(`Default`, 'r', true, || {
            s::open_menu(`ClosureCounter`);
            let count = block();
            s::open_menu(`Node`);
            let node = block();
            for _i in 0..count {
                print(`  Inserting node of type ${node}`);
            }
        });
        s::bind_key(`Default`, 'q', true, || s::escape());
        s::bind_key(`Default`, 'e', true, || s::exit());

        s::bind_key(`Default`, 'n', true, || s::open_menu(`RuntimeCounter`));

        // ClosureCounter Menu
        let count = 1;
        s::bind_key(`ClosureCounter`, 'a', false, || {
            print(`  a pressed ${count} times`);
            count += 1;
        });
        s::bind_key(`ClosureCounter`, 'd', true, || count);
        s::bind_key(`ClosureCounter`, 'q', true, || s::escape());
        s::bind_key(`ClosureCounter`, 'e', true, || s::exit());

        // RuntimeCounter Menu
        s::bind_key(`RuntimeCounter`, '+', false, || {
            print(s::increment())
        });
        s::bind_key(`RuntimeCounter`, '-', false, || {
            allow_error( || print(s::decrement()));
        });
        s::bind_key(`RuntimeCounter`, 'm', false, || {
            print(s::decrement())
        });
        s::bind_key(`RuntimeCounter`, 'd', true, || s::count());
        s::bind_key(`RuntimeCounter`, 'q', true, || s::escape());
        s::bind_key(`RuntimeCounter`, 'x', true, || s::open_menu(`BadMenu`));
        s::bind_key(`RuntimeCounter`, 'X', true, || allow_error(|| s::open_menu(`BadMenu`)));
        s::bind_key(`RuntimeCounter`, '!', true, || [][17]);

        // Node Selection Menu
        s::bind_key(`Node`, 'a', true, || `Array`);
        s::bind_key(`Node`, 'q', true, || s::escape());
        s::bind_key(`Node`, 'e', true, || s::exit());
    ";

    let main_script = "
        loop {
            try {
                s::block(); // ignoring return value
            } catch (err) {
                if type_of(err) == `EditorError` {
                    let msg = err.message;
                    print(`  Editor Error: ${msg}!`);
                } else {
                    let msg = if `message` in err {
                        err.message
                    } else {
                        err
                    };
                    print(`  Exception: ${msg}!`);
                }
                s::close_menu();
            }
        }
    ";

    let prelude_ast = engine.compile(prelude_script).unwrap();
    let mut prelude_module = Module::eval_ast_as_new(Scope::new(), &prelude_ast, &engine).unwrap();
    register_runtime_methods(&mut prelude_module);
    engine.register_static_module("s", prelude_module.into());
    engine.set_strict_variables(true);

    engine.run(init_script).unwrap();
    engine.run(main_script).unwrap();
}
