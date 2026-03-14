// Compiled module C wrappers

use std::slice;

use crate::compiler::deserialize_module;

use super::error::set_error;

#[repr(C)]
pub struct KiraModule {
    _private: [u8; 0],
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_module_from_bytes(
    bytes: *const u8,
    len: usize,
    err: *mut super::error::KiraError,
) -> *mut KiraModule {
    if bytes.is_null() {
        set_error(err, "module bytes are null");
        return std::ptr::null_mut();
    }
    let slice = unsafe { slice::from_raw_parts(bytes, len) };
    match deserialize_module(slice) {
        Ok(module) => Box::into_raw(Box::new(module)) as *mut KiraModule,
        Err(message) => {
            set_error(err, message);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_module_free(module: *mut KiraModule) {
    if module.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(module as *mut crate::compiler::CompiledModule);
    }
}
