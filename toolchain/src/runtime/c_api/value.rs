// C value helpers

use std::ffi::c_void;

use crate::runtime::Value;

use super::error::set_error;

#[repr(C)]
pub struct KiraValue {
    _private: [u8; 0],
}

fn into_value_ptr(value: Value) -> *mut KiraValue {
    Box::into_raw(Box::new(value)) as *mut KiraValue
}

unsafe fn value_ref<'a>(value: *const KiraValue) -> &'a Value {
    unsafe { &*(value as *const Value) }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_unit() -> *mut KiraValue {
    into_value_ptr(Value::Unit)
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_from_int(value: i64) -> *mut KiraValue {
    into_value_ptr(Value::Int(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_from_bool(value: bool) -> *mut KiraValue {
    into_value_ptr(Value::Bool(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_from_float(value: f64) -> *mut KiraValue {
    into_value_ptr(Value::from_f64(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_from_handle_take(handle: *mut c_void) -> *mut KiraValue {
    handle as *mut KiraValue
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_into_handle(value: *mut KiraValue) -> *mut c_void {
    value as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_into_handle_clone(value: *const KiraValue) -> *mut c_void {
    if value.is_null() {
        return std::ptr::null_mut();
    }
    let cloned = unsafe { value_ref(value).clone() };
    Box::into_raw(Box::new(cloned)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_as_int(
    value: *const KiraValue,
    err: *mut super::error::KiraError,
) -> i64 {
    if value.is_null() {
        set_error(err, "expected int value but got null");
        return 0;
    }
    match unsafe { value_ref(value) } {
        Value::Int(v) => *v,
        other => {
            set_error(err, format!("expected int value, got {other:?}"));
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_as_bool(
    value: *const KiraValue,
    err: *mut super::error::KiraError,
) -> bool {
    if value.is_null() {
        set_error(err, "expected bool value but got null");
        return false;
    }
    match unsafe { value_ref(value) } {
        Value::Bool(v) => *v,
        other => {
            set_error(err, format!("expected bool value, got {other:?}"));
            false
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_as_float(
    value: *const KiraValue,
    err: *mut super::error::KiraError,
) -> f64 {
    if value.is_null() {
        set_error(err, "expected float value but got null");
        return 0.0;
    }
    match unsafe { value_ref(value) } {
        Value::Float(v) => v.0,
        other => {
            set_error(err, format!("expected float value, got {other:?}"));
            0.0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_free(value: *mut KiraValue) {
    if value.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(value as *mut Value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_debug_string(
    value: *const KiraValue,
    err: *mut super::error::KiraError,
) -> *mut std::ffi::c_char {
    if value.is_null() {
        set_error(err, "expected value but got null");
        return std::ptr::null_mut();
    }
    let text = format!("{:?}", unsafe { value_ref(value) });
    let cstr = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("<invalid>").unwrap());
    cstr.into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_value_debug_string_free(ptr: *mut std::ffi::c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = std::ffi::CString::from_raw(ptr);
    }
}
