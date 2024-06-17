#[macro_export]
macro_rules! gfx_debug_exec {
    ($a:expr) => {
        if cfg!(feature = "gfx_debug_msg") {
            $a;
        }
    };
}

#[macro_export]
macro_rules! gfx_debug_log {
    ($($arg:tt)+) => {
        crate::gfx_debug_exec!(log::debug!(target: "rust_engine::graphics", $($arg)+))
    };
}
