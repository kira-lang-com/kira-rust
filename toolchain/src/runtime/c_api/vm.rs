// VM C API

use std::ffi::CStr;
use std::path::Path;
use std::slice;

use crate::compiler::CompiledModule;
use crate::runtime::ffi_loader::FfiLoader;
use crate::runtime::vm::{RuntimeError, Vm};
use crate::runtime::Value;

use super::error::{set_error, take_error_message, KiraError};
use super::{KiraModule, KiraNativeHandler, KiraValue};

#[repr(C)]
pub struct KiraVm {
    _private: [u8; 0],
}

fn module_ref<'a>(module: *const KiraModule) -> &'a CompiledModule {
    unsafe { &*(module as *const CompiledModule) }
}

fn vm_mut<'a>(vm: *mut KiraVm) -> &'a mut Vm {
    unsafe { &mut *(vm as *mut Vm) }
}

fn take_value(ptr: *mut KiraValue) -> Result<Value, RuntimeError> {
    if ptr.is_null() {
        return Err(RuntimeError("null value pointer passed to VM".to_string()));
    }
    Ok(unsafe { *Box::from_raw(ptr as *mut Value) })
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_new() -> *mut KiraVm {
    Box::into_raw(Box::new(Vm::default())) as *mut KiraVm
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_free(vm: *mut KiraVm) {
    if vm.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(vm as *mut Vm);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_prepare(
    vm: *mut KiraVm,
    module: *const KiraModule,
    err: *mut KiraError,
) -> bool {
    if vm.is_null() || module.is_null() {
        set_error(err, "vm or module is null");
        return false;
    }
    let module = module_ref(module);
    if module.ffi.functions.is_empty() && module.ffi.links.is_empty() {
        return true;
    }

    let mut loader = FfiLoader::new();
    if let Err(message) = loader.load_ffi_metadata(&module.ffi, Path::new(".")) {
        set_error(err, message);
        return false;
    }
    vm_mut(vm).load_ffi(loader);
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_register_native(
    vm: *mut KiraVm,
    name: *const std::ffi::c_char,
    handler: KiraNativeHandler,
) {
    if vm.is_null() || name.is_null() {
        return;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    vm_mut(vm).register_native(name, handler);
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_run_entry(
    vm: *mut KiraVm,
    module: *const KiraModule,
    entry: *const std::ffi::c_char,
    err: *mut KiraError,
) -> bool {
    if vm.is_null() || module.is_null() || entry.is_null() {
        set_error(err, "vm, module, or entry is null");
        return false;
    }
    let entry = unsafe { CStr::from_ptr(entry) }.to_string_lossy().into_owned();
    match vm_mut(vm).run_entry(module_ref(module), &entry) {
        Ok(_) => true,
        Err(RuntimeError(message)) => {
            set_error(err, message);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_run_function(
    vm: *mut KiraVm,
    module: *const KiraModule,
    name: *const std::ffi::c_char,
    args: *const *mut KiraValue,
    argc: usize,
    out: *mut *mut KiraValue,
    err: *mut KiraError,
) -> bool {
    if vm.is_null() || module.is_null() || name.is_null() {
        set_error(err, "vm, module, or name is null");
        return false;
    }

    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let args = if argc == 0 {
        Vec::new()
    } else if args.is_null() {
        set_error(err, "args pointer is null");
        return false;
    } else {
        let slice = unsafe { slice::from_raw_parts(args, argc) };
        let mut values = Vec::with_capacity(argc);
        for &ptr in slice {
            match take_value(ptr) {
                Ok(value) => values.push(value),
                Err(RuntimeError(message)) => {
                    set_error(err, message);
                    return false;
                }
            }
        }
        values
    };

    match vm_mut(vm).run_function(module_ref(module), &name, args) {
        Ok(result) => {
            if out.is_null() {
                return true;
            }
            unsafe {
                *out = Box::into_raw(Box::new(result)) as *mut KiraValue;
            }
            true
        }
        Err(RuntimeError(message)) => {
            set_error(err, message);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kira_vm_print_output(vm: *mut KiraVm) {
    if vm.is_null() {
        return;
    }
    let vm = vm_mut(vm);
    for line in vm.output() {
        println!("{}", line);
    }
}

pub(crate) fn call_native_handler(
    vm: &mut Vm,
    module: &CompiledModule,
    handler: KiraNativeHandler,
    args: &mut [Value],
) -> Result<Value, RuntimeError> {
    let mut error = KiraError::default();
    let arg_ptrs = args
        .iter_mut()
        .map(|value| value as *mut Value as *const KiraValue)
        .collect::<Vec<_>>();
    let result_ptr = handler(
        vm as *mut Vm as *mut KiraVm,
        module as *const CompiledModule as *const KiraModule,
        arg_ptrs.as_ptr(),
        args.len(),
        &mut error,
    );
    if let Some(message) = take_error_message(&mut error) {
        return Err(RuntimeError(message));
    }
    if result_ptr.is_null() {
        return Err(RuntimeError("native handler returned null".to_string()));
    }
    Ok(unsafe { *Box::from_raw(result_ptr as *mut Value) })
}
