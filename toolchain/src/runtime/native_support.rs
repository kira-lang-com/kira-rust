use std::ffi::c_void;
use std::slice;

use ordered_float::OrderedFloat;

use crate::compiler::CompiledModule;

use super::{
    type_system::KiraType,
    value::StructValue,
    vm::Vm,
    Value,
};

#[repr(C)]
struct NativeRuntimeContext {
    vm: *mut Vm,
    module: *const CompiledModule,
}

fn into_handle(value: Value) -> *mut c_void {
    Box::into_raw(Box::new(value)) as *mut c_void
}

unsafe fn clone_handle_value(handle: *mut c_void) -> Value {
    unsafe { (*(handle as *mut Value)).clone() }
}

unsafe fn take_handle_value(handle: *mut c_void) -> Value {
    unsafe { *Box::from_raw(handle as *mut Value) }
}

unsafe fn ctx_module<'a>(ctx: *mut c_void) -> &'a CompiledModule {
    unsafe { &*(*(ctx as *mut NativeRuntimeContext)).module }
}

unsafe fn ctx_vm<'a>(ctx: *mut c_void) -> &'a mut Vm {
    unsafe { &mut *(*(ctx as *mut NativeRuntimeContext)).vm }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_box_int(value: i64) -> *mut c_void {
    into_handle(Value::Int(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_box_bool(value: bool) -> *mut c_void {
    into_handle(Value::Bool(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_box_float(value: f64) -> *mut c_void {
    into_handle(Value::Float(OrderedFloat(value)))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_unbox_int(handle: *mut c_void) -> i64 {
    unsafe {
        match take_handle_value(handle) {
            Value::Int(value) => value,
            other => panic!("expected int handle, got {:?}", other),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_unbox_bool(handle: *mut c_void) -> bool {
    unsafe {
        match take_handle_value(handle) {
            Value::Bool(value) => value,
            other => panic!("expected bool handle, got {:?}", other),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_unbox_float(handle: *mut c_void) -> f64 {
    unsafe {
        match take_handle_value(handle) {
            Value::Float(value) => value.0,
            other => panic!("expected float handle, got {:?}", other),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_clone_value(handle: *mut c_void) -> *mut c_void {
    unsafe { into_handle(clone_handle_value(handle)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_make_string(bytes: *const u8, len: usize) -> *mut c_void {
    let bytes = unsafe { slice::from_raw_parts(bytes, len) };
    let value = String::from_utf8(bytes.to_vec()).expect("string constants must be utf-8");
    into_handle(Value::String(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_value_eq(left: *mut c_void, right: *mut c_void) -> bool {
    unsafe { clone_handle_value(left) == clone_handle_value(right) }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_print_int(ctx: *mut c_void, value: i64) {
    unsafe { ctx_vm(ctx).output.push(Value::Int(value).display()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_print_bool(ctx: *mut c_void, value: bool) {
    unsafe { ctx_vm(ctx).output.push(Value::Bool(value).display()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_print_float(ctx: *mut c_void, value: f64) {
    unsafe {
        ctx_vm(ctx)
            .output
            .push(Value::Float(OrderedFloat(value)).display())
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_print_value(ctx: *mut c_void, value: *mut c_void) {
    unsafe { ctx_vm(ctx).output.push(clone_handle_value(value).display()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_new_array() -> *mut c_void {
    into_handle(Value::Array(Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_array_push(array: *mut c_void, value: *mut c_void) {
    unsafe {
        let Value::Array(elements) = &mut *(array as *mut Value) else {
            panic!("expected array handle");
        };
        elements.push(take_handle_value(value));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_array_length(array: *mut c_void) -> i64 {
    unsafe {
        let Value::Array(elements) = &*(array as *mut Value) else {
            panic!("expected array handle");
        };
        elements.len() as i64
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_array_index(array: *mut c_void, index: i64) -> *mut c_void {
    unsafe {
        let Value::Array(elements) = &*(array as *mut Value) else {
            panic!("expected array handle");
        };
        let index = usize::try_from(index).expect("array index must be non-negative");
        into_handle(
            elements
                .get(index)
                .unwrap_or_else(|| panic!("array index {} out of bounds", index))
                .clone(),
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_array_append(array: *mut c_void, value: *mut c_void) {
    kira_native_array_push(array, value);
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_new_struct(ctx: *mut c_void, type_id: usize) -> *mut c_void {
    unsafe {
        let module = ctx_module(ctx);
        let type_id = crate::runtime::type_system::TypeId(type_id);
        let KiraType::Struct(struct_type) = module.types.get(type_id) else {
            panic!("type id {} is not a struct", type_id.0);
        };
        into_handle(Value::Struct(StructValue {
            type_name: struct_type.name.clone(),
            fields: struct_type
                .fields
                .iter()
                .map(|field| (field.name.clone(), Value::Unit))
                .collect(),
        }))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_struct_set_field(
    target: *mut c_void,
    index: usize,
    value: *mut c_void,
) {
    unsafe {
        let Value::Struct(struct_value) = &mut *(target as *mut Value) else {
            panic!("expected struct handle");
        };
        let (_, field_value) = struct_value
            .fields
            .get_mut(index)
            .unwrap_or_else(|| panic!("invalid struct field index {}", index));
        *field_value = take_handle_value(value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_struct_field(target: *mut c_void, index: usize) -> *mut c_void {
    unsafe {
        let Value::Struct(struct_value) = &*(target as *mut Value) else {
            panic!("expected struct handle");
        };
        into_handle(
            struct_value
                .fields
                .get(index)
                .unwrap_or_else(|| panic!("invalid struct field index {}", index))
                .1
                .clone(),
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_native_store_struct_field(
    target: *mut c_void,
    path: *const usize,
    path_len: usize,
    value: *mut c_void,
) {
    unsafe {
        let path = slice::from_raw_parts(path, path_len);
        let value = take_handle_value(value);
        store_struct_field(&mut *(target as *mut Value), path, value);
    }
}

fn store_struct_field(target: &mut Value, path: &[usize], value: Value) {
    let Some((field_index, rest)) = path.split_first() else {
        *target = value;
        return;
    };

    let Value::Struct(struct_value) = target else {
        panic!("expected struct while storing nested field");
    };
    let (_, field_value) = struct_value
        .fields
        .get_mut(*field_index)
        .unwrap_or_else(|| panic!("invalid struct field index {}", field_index));

    if rest.is_empty() {
        *field_value = value;
    } else {
        store_struct_field(field_value, rest, value);
    }
}
