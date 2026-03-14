use std::collections::HashMap;
use std::path::Path;

use libloading::{Library, Symbol};

use crate::compiler::FfiMetadata;
use crate::runtime::{Value, vm::RuntimeError};

pub struct FfiLoader {
    libraries: HashMap<String, Library>,
    function_to_library: HashMap<String, String>,
}

impl FfiLoader {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
            function_to_library: HashMap::new(),
        }
    }

    pub fn load_ffi_metadata(&mut self, ffi: &FfiMetadata, _project_root: &Path) -> Result<(), String> {
        for link in &ffi.links {
            // Try to load the library from search paths
            let mut loaded = false;
            
            for search_path in &link.search_paths {
                let lib_path = Path::new(search_path).join(format_lib_name(&link.library));
                
                if lib_path.exists() {
                    match unsafe { Library::new(&lib_path) } {
                        Ok(lib) => {
                            self.libraries.insert(link.library.clone(), lib);
                            loaded = true;
                            break;
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load library from {}: {}", lib_path.display(), e);
                        }
                    }
                }
            }
            
            // Try loading from system paths if not found in search paths
            if !loaded {
                let lib_name = format_lib_name(&link.library);
                match unsafe { Library::new(&lib_name) } {
                    Ok(lib) => {
                        self.libraries.insert(link.library.clone(), lib);
                    }
                    Err(e) => {
                        return Err(format!(
                            "Failed to load library '{}': {}. Search paths: {:?}",
                            link.library, e, link.search_paths
                        ));
                    }
                }
            }
        }
        
        // Build a map from function names to library names
        // For now, we'll try to find functions in all loaded libraries
        for (func_name, _ffi_func) in &ffi.functions {
            // Try to find which library has this function
            for (lib_name, library) in &self.libraries {
                unsafe {
                    if library.get::<Symbol<unsafe extern "C" fn()>>(func_name.as_bytes()).is_ok() {
                        self.function_to_library.insert(func_name.clone(), lib_name.clone());
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn call_ffi_function(
        &self,
        function_name: &str,
        args: Vec<Value>,
        ffi: &FfiMetadata,
    ) -> Result<Value, RuntimeError> {
        // Find which library contains this function
        let _ffi_func = ffi.functions.get(function_name).ok_or_else(|| {
            RuntimeError(format!("FFI function '{}' not found in metadata", function_name))
        })?;

        let lib_name = self.function_to_library.get(function_name).ok_or_else(|| {
            RuntimeError(format!("Library for function '{}' not found", function_name))
        })?;

        let library = self.libraries.get(lib_name).ok_or_else(|| {
            RuntimeError(format!("Library '{}' not loaded", lib_name))
        })?;

        // Get the function symbol
        let symbol: Symbol<unsafe extern "C" fn() -> i64> = unsafe {
            library.get(function_name.as_bytes()).map_err(|e| {
                RuntimeError(format!("Failed to get symbol '{}': {}", function_name, e))
            })?
        };

        // Call the function based on signature
        // This is a simplified implementation - you'll need to handle different signatures
        let result = unsafe {
            match args.len() {
                0 => {
                    let func: Symbol<unsafe extern "C" fn() -> i64> = std::mem::transmute(symbol);
                    func()
                }
                1 => {
                    let func: Symbol<unsafe extern "C" fn(i64) -> i64> = std::mem::transmute(symbol);
                    let arg0 = value_to_i64(&args[0])?;
                    func(arg0)
                }
                2 => {
                    let func: Symbol<unsafe extern "C" fn(i64, i64) -> i64> = std::mem::transmute(symbol);
                    let arg0 = value_to_i64(&args[0])?;
                    let arg1 = value_to_i64(&args[1])?;
                    func(arg0, arg1)
                }
                3 => {
                    let func: Symbol<unsafe extern "C" fn(i64, i64, i64) -> i64> = std::mem::transmute(symbol);
                    let arg0 = value_to_i64(&args[0])?;
                    let arg1 = value_to_i64(&args[1])?;
                    let arg2 = value_to_i64(&args[2])?;
                    func(arg0, arg1, arg2)
                }
                _ => {
                    return Err(RuntimeError(format!(
                        "FFI functions with {} arguments not yet supported",
                        args.len()
                    )));
                }
            }
        };

        Ok(Value::Int(result))
    }
}

fn format_lib_name(name: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        if name.starts_with("lib") && name.ends_with(".dylib") {
            name.to_string()
        } else if name.starts_with("lib") {
            format!("{}.dylib", name)
        } else {
            format!("lib{}.dylib", name)
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if name.starts_with("lib") && name.ends_with(".so") {
            name.to_string()
        } else if name.starts_with("lib") {
            format!("{}.so", name)
        } else {
            format!("lib{}.so", name)
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if name.ends_with(".dll") {
            name.to_string()
        } else {
            format!("{}.dll", name)
        }
    }
}

fn value_to_i64(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Int(i) => Ok(*i),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => Err(RuntimeError(format!(
            "Cannot convert {:?} to FFI argument",
            value
        ))),
    }
}
