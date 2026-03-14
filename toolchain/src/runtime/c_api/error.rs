// C error helpers

use std::ffi::{CStr, CString, c_char};
use std::ptr;

#[repr(C)]
pub struct KiraError {
    pub message: *mut c_char,
}

impl Default for KiraError {
    fn default() -> Self {
        Self {
            message: ptr::null_mut(),
        }
    }
}

pub(crate) fn set_error(err: *mut KiraError, message: impl Into<String>) {
    if err.is_null() {
        return;
    }
    unsafe {
        clear_error(err);
        let text = message.into();
        let cstr = CString::new(text).unwrap_or_else(|_| CString::new("error").unwrap());
        (*err).message = cstr.into_raw();
    }
}

pub(crate) fn clear_error(err: *mut KiraError) {
    if err.is_null() {
        return;
    }
    unsafe {
        if !(*err).message.is_null() {
            let _ = CString::from_raw((*err).message);
            (*err).message = ptr::null_mut();
        }
    }
}

pub(crate) fn take_error_message(err: &mut KiraError) -> Option<String> {
    if err.message.is_null() {
        return None;
    }
    unsafe {
        let cstr = CString::from_raw(err.message);
        err.message = ptr::null_mut();
        Some(cstr.to_string_lossy().into_owned())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_error_has(err: *const KiraError) -> bool {
    if err.is_null() {
        return false;
    }
    unsafe { !(*err).message.is_null() }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_error_set(err: *mut KiraError, message: *const c_char) {
    if message.is_null() {
        set_error(err, "unknown error");
        return;
    }
    let text = unsafe { CStr::from_ptr(message) };
    set_error(err, text.to_string_lossy().into_owned());
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_error_free(err: *mut KiraError) {
    clear_error(err);
}
