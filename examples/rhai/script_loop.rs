use rhai::{CustomType, Engine, FnPtr, FuncRegistration, Module, Scope, TypeBuilder};
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

#[derive(Debug, Default)]
struct Runtime {
    keymaps: HashMap<String, HashMap<char, KeyProg>>,
    active_menu: String,
}

#[derive(Debug, Clone)]
struct KeyProg {
    prog: FnPtr,
    close_menu: bool,
}

impl CustomType for KeyProg {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Runtime")
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

    fn enter_menu(&mut self, menu: &str) {
        self.active_menu = menu.to_owned();
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
}

macro_rules! register {
    ($module:expr, $fn_name:literal, $closure:expr) => {
        FuncRegistration::new($fn_name)
            .in_internal_namespace()
            .set_into_module($module, $closure);
    };
}

fn register_runtime_methods(module: &mut Module) {
    let runtime = Rc::new(RefCell::new(Runtime {
        keymaps: HashMap::new(),
        active_menu: "Default".to_owned(),
    }));

    let rt = runtime.clone();
    register!(module, "enter_menu", move |menu: &str| {
        rt.borrow_mut().enter_menu(menu)
    });

    let rt = runtime.clone();
    register!(module, "close_menu", move || rt.borrow_mut().close_menu());

    let rt = runtime.clone();
    register!(
        module,
        "bind_key",
        move |keymap: &str, key: char, close_menu: bool, prog: FnPtr| {
            rt.borrow_mut().bind_key(keymap, key, close_menu, prog)
        }
    );

    let rt = runtime.clone();
    register!(module, "block_for_keyprog", move || {
        rt.borrow_mut().block_for_keyprog()
    });

    let rt = runtime.clone();
    register!(module, "exit", move || rt.borrow_mut().exit());
}

pub fn main() {
    let mut engine = Engine::new();

    engine.build_type::<KeyProg>();

    println!("Signatures:");
    engine
        .gen_fn_signatures(false)
        .into_iter()
        .for_each(|func| println!("  {func}"));
    println!();

    let prelude_script = "
        fn block() {
            let keyprog = s::block_for_keyprog();
            if keyprog.close_menu {
                s::close_menu();
            }
            call(keyprog.prog)
        }
        
        fn escape() {
            throw `escape`;
        }
    ";

    let init_script = "
        // Default Menu
        s::bind_key(`Default`, 'c', true, || s::enter_menu(`Counter`));
        s::bind_key(`Default`, 'i', true, || {
            s::enter_menu(`Node`);
            let node = block();
            // return to main loop
            print(`  Inserting node of type ${node}`);
        });
        s::bind_key(`Default`, 'q', true, || s::escape());
        s::bind_key(`Default`, 'e', true, || s::exit());

        // Counter Menu
        let count = 1;
        s::bind_key(`Counter`, 'a', false, || {
            print(`  a pressed ${count} times`);
            count += 1;
        });
        s::bind_key(`Counter`, 'q', true, || s::escape());
        s::bind_key(`Counter`, 'e', true, || s::exit());

        // Node Selection Menu
        s::bind_key(`Node`, 'a', true, || `Array`);
        s::bind_key(`Node`, 'q', true, || s::escape());
        s::bind_key(`Node`, 'e', true, || s::exit());
    ";

    let main_script = "
        loop {
            try {
                s::block(); // ignoring return value
            } catch (exc) {
                print(`  Exception ${exc}!`);
                s::close_menu();
            }
        }
    ";

    let prelude_ast = engine.compile(prelude_script).unwrap();
    let mut prelude_module = Module::eval_ast_as_new(Scope::new(), &prelude_ast, &engine).unwrap();
    register_runtime_methods(&mut prelude_module);
    engine.register_static_module("s", prelude_module.into());

    engine.run(init_script).unwrap();
    engine.run(main_script).unwrap();
}
