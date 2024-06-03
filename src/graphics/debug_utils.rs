use super::instance::Instance;
use ash::vk;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::panic::{catch_unwind, AssertUnwindSafe, RefUnwindSafe};
use std::rc::Rc;

pub struct DebugUtils {
    debug_instance: ash::ext::debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    instance: Rc<Instance>,
    debug_utils_callback: Option<DebugUtilsCallback>,
}

impl DebugUtils {
    pub(self) fn new(
        debug_instance: ash::ext::debug_utils::Instance,
        debug_utils_messenger: vk::DebugUtilsMessengerEXT,
        instance: Rc<Instance>,
        debug_utils_callback: Option<DebugUtilsCallback>,
    ) -> Self {
        Self {
            debug_instance,
            debug_utils_messenger,
            instance,
            debug_utils_callback,
        }
    }
}

impl Debug for DebugUtils {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugUtils")
            .field("debug_instance", &std::ptr::addr_of!(self.debug_instance))
            .field("debug_utils_messenger", &self.debug_utils_messenger)
            .field("instance", &self.instance)
            .field("debug_utils_callback", &self.debug_utils_callback)
            .finish()
    }
}

impl Drop for DebugUtils {
    fn drop(&mut self) {
        println!(stringify!(DebugUtils::drop()));
        unsafe {
            self.debug_instance
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
        }
    }
}

type CallbackFn = Box<
    dyn Fn(
            vk::DebugUtilsMessageSeverityFlagsEXT,
            vk::DebugUtilsMessageTypeFlagsEXT,
            DebugUtilsMessengerCallbackData<'_>,
        ) + RefUnwindSafe
        + Send
        + Sync,
>;

#[non_exhaustive]
pub struct DebugUtilsMessengerCallbackData<'a> {
    pub message_id_name: Option<&'a str>,
    pub message_id_number: i32,
    pub message: &'a str,
}

impl<'a> From<vk::DebugUtilsMessengerCallbackDataEXT<'_>> for DebugUtilsMessengerCallbackData<'a> {
    fn from(value: vk::DebugUtilsMessengerCallbackDataEXT<'_>) -> Self {
        let vk::DebugUtilsMessengerCallbackDataEXT {
            p_message_id_name,
            message_id_number,
            p_message,
            ..
        } = value;

        unsafe {
            Self {
                message_id_name: p_message_id_name
                    .as_ref()
                    .map(|t| std::ffi::CStr::from_ptr(t).to_str().unwrap()),
                message_id_number,
                message: std::ffi::CStr::from_ptr(p_message).to_str().unwrap(),
            }
        }
    }
}

pub struct DebugUtilsCallback(CallbackFn);

impl Debug for DebugUtilsCallback {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugUtilsCallback")
            .field("0", &self.as_ptr())
            .finish()
    }
}

impl DebugUtilsCallback {
    pub unsafe fn new<T>(func: T) -> Self
    where
        T: Fn(
                vk::DebugUtilsMessageSeverityFlagsEXT,
                vk::DebugUtilsMessageTypeFlagsEXT,
                DebugUtilsMessengerCallbackData<'_>,
            ) + RefUnwindSafe
            + Send
            + Sync
            + 'static,
    {
        Self(Box::new(func))
    }

    pub(self) fn as_ptr(&self) -> *const CallbackFn {
        std::ptr::addr_of!(self.0)
    }
}

#[derive(Debug)]
pub struct DebugUtilsBuilder {
    pub message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    pub message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    pub debug_utils_callback: Option<DebugUtilsCallback>,
}

impl Default for DebugUtilsBuilder {
    fn default() -> Self {
        Self {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            debug_utils_callback: None,
        }
    }
}

impl DebugUtilsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message_severity(
        mut self,
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    ) -> Self {
        self.message_severity = message_severity;
        self
    }

    pub fn message_type(mut self, message_type: vk::DebugUtilsMessageTypeFlagsEXT) -> Self {
        self.message_type = message_type;
        self
    }

    pub fn debug_utils_callback(mut self, debug_utils_callback: DebugUtilsCallback) -> Self {
        self.debug_utils_callback = Some(debug_utils_callback);
        self
    }

    pub fn build(self, instance: Rc<Instance>) -> ash::prelude::VkResult<DebugUtils> {
        let user_date = match self.debug_utils_callback.as_ref() {
            None => std::ptr::null_mut(),
            Some(data) => data.as_ptr().cast_mut().cast(),
        };

        let debug_utils_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(self.message_severity)
            .message_type(self.message_type)
            .pfn_user_callback(Some(raw_debug_callback))
            .user_data(user_date);

        let debug_instance =
            ash::ext::debug_utils::Instance::new(&instance.entry(), &instance.handle());
        let debug_utils_messenger =
            unsafe { debug_instance.create_debug_utils_messenger(&debug_utils_info, None)? };

        Ok(DebugUtils::new(
            debug_instance,
            debug_utils_messenger,
            instance,
            self.debug_utils_callback,
        ))
    }
}

pub(self) unsafe extern "system" fn raw_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _: *mut c_void,
) -> vk::Bool32 {
    let _ = catch_unwind(AssertUnwindSafe(move || {
        let raw_callback_data = *p_callback_data;

        let data = DebugUtilsMessengerCallbackData::from(raw_callback_data);

        println!(
            "[{:?}::{:?}]: {}",
            message_severity, message_types, data.message
        );

        // let user_callback: &CallbackFn = &*p_user_data.cast_const().cast();

        // user_callback(message_severity, message_types, data);
    }));

    vk::FALSE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::instance::InstanceBuilder;

    fn create_instance() -> Rc<Instance> {
        InstanceBuilder::new()
            .extensions(vec![ash::ext::debug_utils::NAME
                .to_str()
                .unwrap()
                .to_string()])
            .layers(vec!["VK_LAYER_KHRONOS_validation".to_owned()])
            .api_version(vk::API_VERSION_1_3)
            .build()
            .unwrap()
    }

    #[test]
    fn test_create_debug_utils() {
        let instance = create_instance();

        let debug_utils = DebugUtilsBuilder::new().build(instance.clone());

        assert!(debug_utils.is_ok());
    }

    #[test]
    fn test_create_debug_utils_without_callback() {
        let instance = create_instance();

        let debug_utils = DebugUtilsBuilder::new().build(instance.clone());

        assert!(debug_utils.is_ok());
    }

    #[test]
    fn test_debug_format() {
        let instance = create_instance();

        let debug_utils = DebugUtilsBuilder::new().build(instance.clone()).unwrap();

        let debug_utils_string = format!("{:?}", debug_utils);

        println!("{debug_utils_string}");
    }
}
