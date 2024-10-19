let args = s::cli_args();
if "file_path" in args {
    try {
        s::open_doc(args["file_path"])
    } catch (err) {
        s::log_caught_error(err);
    }
}

loop {
    try {
        s::block(); // ignoring return value
    } catch (err) {
        if type_of(err) == "SynlessError" {
            switch err.category {
                "Abort" => {
                    s::log_caught_error(err);
                    s::abort();
                }
                "Escape" => {
                    s::log_debug("escaped");
                }
                _ => {
                    s::log_caught_error(err);
                }
            }
        } else {
            s::log_caught_error(err);
        }

        s::close_menu();
    }
}
