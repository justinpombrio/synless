#[macro_export]
macro_rules! error {
    ($msg:expr) => ({
        my_panic!($msg,)
    });
    ($fmt:expr, $($arg:tt),*) => ({
        let msg = concat!(
            "Internal error in Synless!\n",
            $fmt);
        panic!(msg, $($arg),*);
    });
}

#[macro_export]
macro_rules! expect {
    ($result:expr, $msg:expr) => {{
        let msg = concat!("Internal error in Synless!\n", $msg);
        $result.expect(msg)
    }};
}
