#[macro_export]
macro_rules! debug_exec {
    ($a:expr) => {
        if cfg!(feature = "gfx_debug_msg") {
            $a
        }
    };
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)+) => {
        crate::debug_exec!(log::debug!(target: "rust_engine", $($arg)+))
    };
}
