use std::fmt::{Debug, Formatter};

pub struct DebugUtils {
    debug_instance: ash::ext::debug_utils::Instance,
}

impl DebugUtils {

}

impl Debug for DebugUtils {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugUtils")
            .finish()
    }
}