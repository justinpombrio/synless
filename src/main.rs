use std::cell::RefCell;
use std::panic;
use std::rc::Rc;
use synless::{log, ColorTheme, Log, Runtime, Settings, SynlessBug, SynlessError, Terminal};

// TODO: Make this work if you start in a different cwd
const BASE_MODULE_PATH: &str = "scripts/base_module.rhai";
const INTERNALS_MODULE_PATH: &str = "scripts/internals_module.rhai";
const INIT_PATH: &str = "scripts/init.rhai";
const MAIN_PATH: &str = "scripts/main.rhai";

fn make_engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_fail_on_invalid_map_property(true);
    engine.on_print(|msg| log!(Info, "{msg}"));
    engine.on_debug(|msg, src, pos| {
        let src = src.unwrap_or("unknown");
        log!(Debug, "{src} @ {pos:?} > {msg}");
    });

    engine.build_type::<synless::Keymap>();
    engine.build_type::<synless::Layer>();
    engine.build_type::<synless::KeyProg>();
    engine.build_type::<synless::SynlessError>();

    println!("Signatures:");
    engine
        .gen_fn_signatures(false)
        .into_iter()
        .for_each(|func| println!("  {func}"));
    println!();

    engine
}

fn make_runtime() -> Rc<RefCell<Runtime<Terminal>>> {
    let settings = Settings::default();
    let terminal =
        Terminal::new(ColorTheme::default_dark()).bug_msg("Failed to construct terminal frontend");
    let runtime = Runtime::new(settings, terminal);
    Rc::new(RefCell::new(runtime))
}

fn run() -> Result<(), Box<rhai::EvalAltResult>> {
    let mut engine = make_engine();

    // Load internals_module.rhai
    let mut internals_mod = {
        let internals_ast = engine.compile_file(INTERNALS_MODULE_PATH.into())?;
        rhai::Module::eval_ast_as_new(rhai::Scope::new(), &internals_ast, &engine)?
    };

    // Load base_module.rhai
    let mut base_mod = {
        let base_ast = engine.compile_file(BASE_MODULE_PATH.into())?;
        rhai::Module::eval_ast_as_new(rhai::Scope::new(), &base_ast, &engine)?
    };

    // Register runtime methods into internals_module and base_module
    let runtime = make_runtime();
    Runtime::register_internal_methods(runtime.clone(), &mut internals_mod);
    engine.register_static_module("synless_internals", internals_mod.into());
    Runtime::register_external_methods(runtime, &mut base_mod);
    engine.register_static_module("s", base_mod.into());

    // Can't set this before modules are registered, as they reference each other
    engine.set_strict_variables(true);

    // Load init.rhai as a module, so keybindings can call functions defined in it.
    let init_mod = {
        let init_ast = engine.compile_file(INIT_PATH.into())?;
        rhai::Module::eval_ast_as_new(rhai::Scope::new(), &init_ast, &engine)?
    };
    engine.register_global_module(init_mod.into());

    // Load main.rhai
    let main_ast = engine.compile_file(MAIN_PATH.into())?;
    engine.run_ast(&main_ast)?;

    Ok(())
}

fn display_error(error: Box<rhai::EvalAltResult>) {
    if let rhai::EvalAltResult::ErrorRuntime(value, _) = error.as_ref() {
        if let Some(synless_error) = value.clone().try_cast::<SynlessError>() {
            log!(Error, "Uncaught error in main: {synless_error}");
            return;
        }
    }
    log!(Error, "Uncaught error in main: {error}");
}

fn main() {
    log!(Info, "Synless is starting");

    // Set up panic handling. We can't simply print the panic message to stderr,
    // because it would be swallowed by the terminal's alternate screen. Instead,
    // we'll log it and print the log once the terminal has been dropped.
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        let mut message = "Rust panic".to_owned();
        if let Some(location) = info.location() {
            message.push_str(&format!(" @ {location}"));
        }
        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            message.push_str(&format!(": {payload}"));
        }
        log!(Error, "{message}")
    }));

    // Run the editor, catching any panics, then print the log.
    let _ = panic::catch_unwind(|| {
        if let Err(err) = run() {
            display_error(err);
        }
    });
    panic::set_hook(old_hook);
    println!("{}", Log::to_string());
}
