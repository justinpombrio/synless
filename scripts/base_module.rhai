fn block() {
    loop {
        let keyprog = synless_internals::block_on_key();
        if keyprog.close_menu {
            s::close_menu();
            return call(keyprog.prog);
        }
        call(keyprog.prog);
    }
}

fn abort() {
    synless_internals::prepare_to_abort();
    exit();
}
