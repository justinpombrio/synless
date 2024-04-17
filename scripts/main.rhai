loop {
    try {
        s::block(); // ignoring return value
    } catch (err) {
        if type_of(err) == "SynlessError" {
            let category = err.category;
            let msg = err.message;
            s::log_error(`${category}: ${msg}`);
            if err.category == "Abort" {
                s::abort();
            } 
        } else {
            if type_of(err) == "map" && "message" in err {
                let msg = err.message;
                s::log_error(`Rhai: ${msg}`);
            } else {
                s::log_error(`Thrown: ${err}`);
            };
        }
        s::close_menu();
    }
}
