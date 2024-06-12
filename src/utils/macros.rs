#[macro_export]
macro_rules! debug_exec {
    ($a:expr) => {
        if cfg!(debug_assertions) {
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
