loop {
    try {
        s::block(); // ignoring return value
    } catch (err) {
        s::log_caught_error(err);
        if type_of(err) == "SynlessError" && err.category == "Abort" {
            s::abort();
        }
        s::close_menu();
    }
}
