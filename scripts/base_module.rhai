fn block() {
    loop {
        let keyprog = ();
        try {
            synless_internals::display();
            keyprog = synless_internals::block_on_key();
        } catch (err) {
            log_caught_error(err);
            s::abort();
        };
        if keyprog.close_menu {
            s::close_menu();
            return keyprog.prog.call();
        }
        call(keyprog.prog);
    }
}

fn log_caught_error(err) {
    if type_of(err) == "SynlessError" {
        let category = err.category;
        let msg = err.message;
        s::log_error(`${category}: ${msg}`);
    } else {
        if type_of(err) == "map" && "message" in err {
            let msg = err.message;
            s::log_error(`Rhai: ${msg}`);
        } else {
            s::log_error(`Thrown: ${err}`);
        };
    }
}

fn abort() {
    synless_internals::prepare_to_abort();
    exit();
}
