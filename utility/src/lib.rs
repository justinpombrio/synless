#[macro_export]
macro_rules! spanic {
    ($msg:expr) => ({
        spanic!($msg,)
    });
    ($fmt:expr, $($arg:tt),*) => ({
        panic!(
            concat!("Internal error in Synless!\n", $fmt),
            $($arg),*);
    });
}

#[macro_export]
macro_rules! spect {
    ($result:expr, $msg:expr) => {{
        let msg = concat!("Internal error in Synless!\n", $msg);
        $result.expect(msg)
    }};
}
