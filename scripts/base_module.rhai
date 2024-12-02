fn block() {
    loop {
        let keyprog = ();
        try {
            synless_internals::display();
            s::clear_last_log();
            keyprog = synless_internals::block_on_key();
        } catch (err) {
            log_caught_error(err);
            s::abort();
        };
        if keyprog.close_menu {
            s::close_menu();
            return call(keyprog.prog);
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

fn close() {
    if s::has_unsaved_changes() {
        s::log_error(`Doc has unsaved changes. Use ' C' to close without saving.`);
    } else {
        s::force_close_visible_doc();
    }
}

fn quit() {
    while s::has_visible_doc() {
        if s::has_unsaved_changes() {
            s::log_error(`Doc has unsaved changes. Use ' C' to close it without saving, or ' Q' to quit without saving any docs.`);
            return;
        } else {
            s::force_close_visible_doc();
        }
    }
    s::abort();
}
