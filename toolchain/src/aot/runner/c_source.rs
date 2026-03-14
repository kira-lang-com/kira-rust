// C runner source generation

use crate::aot::bridge::collect_runtime_bridges;
use crate::aot::error::AotError;
use crate::aot::runner::c_abi::{c_param_list, c_return_type};
use crate::aot::runner::c_bridges::generate_bridge_function;
use crate::aot::runner::c_wrappers::generate_native_wrapper;
use crate::aot::runner::mangle_ident;
use crate::compiler::{BackendKind, CompiledModule};

pub fn generate_c_runner_source(
    module: &CompiledModule,
    entry_symbol: &str,
) -> Result<String, AotError> {
    let native_functions = module
        .functions
        .values()
        .filter(|function| function.selected_backend == BackendKind::Native)
        .collect::<Vec<_>>();

    let runtime_bridges = collect_runtime_bridges(module)?;

    let mut source = String::new();
    source.push_str("#include <stdbool.h>\n");
    source.push_str("#include <stddef.h>\n");
    source.push_str("#include <stdint.h>\n");
    source.push_str("#include <stdio.h>\n");
    source.push_str("#include <stdlib.h>\n");
    source.push_str("#include <string.h>\n");
    source.push_str("#ifdef _WIN32\n");
    source.push_str("#include <windows.h>\n");
    source.push_str("#else\n");
    source.push_str("#include <limits.h>\n");
    source.push_str("#include <unistd.h>\n");
    source.push_str("#endif\n");
    source.push_str("#ifdef __APPLE__\n");
    source.push_str("#include <mach-o/dyld.h>\n");
    source.push_str("#endif\n\n");

    source.push_str("typedef struct KiraVm KiraVm;\n");
    source.push_str("typedef struct KiraModule KiraModule;\n");
    source.push_str("typedef struct KiraValue KiraValue;\n\n");

    source.push_str("typedef struct KiraError {\n    char *message;\n} KiraError;\n\n");
    source.push_str("typedef KiraValue* (*KiraNativeHandler)(KiraVm*, const KiraModule*, const KiraValue* const*, size_t, KiraError*);\n\n");

    source.push_str("typedef struct NativeRuntimeContext {\n    KiraVm *vm;\n    const KiraModule *module;\n} NativeRuntimeContext;\n\n");

    source.push_str("KiraModule* kira_module_from_bytes(const unsigned char* bytes, size_t len, KiraError* err);\n");
    source.push_str("void kira_module_free(KiraModule* module);\n\n");

    source.push_str("KiraVm* kira_vm_new(void);\n");
    source.push_str("void kira_vm_free(KiraVm* vm);\n");
    source.push_str("bool kira_vm_prepare(KiraVm* vm, const KiraModule* module, KiraError* err);\n");
    source.push_str("void kira_vm_register_native(KiraVm* vm, const char* name, KiraNativeHandler handler);\n");
    source.push_str("bool kira_vm_run_entry(KiraVm* vm, const KiraModule* module, const char* entry, KiraError* err);\n");
    source.push_str("bool kira_vm_run_function(KiraVm* vm, const KiraModule* module, const char* name, KiraValue** args, size_t argc, KiraValue** out, KiraError* err);\n");
    source.push_str("void kira_vm_print_output(KiraVm* vm);\n\n");

    source.push_str("bool kira_error_has(const KiraError* err);\n");
    source.push_str("void kira_error_set(KiraError* err, const char* message);\n");
    source.push_str("void kira_error_free(KiraError* err);\n\n");

    source.push_str("KiraValue* kira_value_unit(void);\n");
    source.push_str("KiraValue* kira_value_from_int(int64_t value);\n");
    source.push_str("KiraValue* kira_value_from_bool(bool value);\n");
    source.push_str("KiraValue* kira_value_from_float(double value);\n");
    source.push_str("KiraValue* kira_value_from_handle_take(void* handle);\n");
    source.push_str("void* kira_value_into_handle(KiraValue* value);\n");
    source.push_str("void* kira_value_into_handle_clone(const KiraValue* value);\n");
    source.push_str("int64_t kira_value_as_int(const KiraValue* value, KiraError* err);\n");
    source.push_str("bool kira_value_as_bool(const KiraValue* value, KiraError* err);\n");
    source.push_str("double kira_value_as_float(const KiraValue* value, KiraError* err);\n");
    source.push_str("void kira_value_free(KiraValue* value);\n\n");
    source.push_str("#ifndef KIRA_MODULE_FILENAME\n");
    source.push_str("#define KIRA_MODULE_FILENAME \"compiled_module.bin\"\n");
    source.push_str("#endif\n\n");
    source.push_str("static void abort_with_error(const char* context, KiraError* err);\n");
    source.push_str("static char* kira_strdup(const char* src);\n");
    source.push_str("static char* kira_executable_path(void);\n");
    source.push_str("static char* kira_module_path(void);\n");
    source.push_str("static bool kira_read_module_bytes(unsigned char** out_bytes, size_t* out_len, KiraError* err);\n\n");

    for function in &native_functions {
        let signature = &function.signature;
        let decl = format!(
            "extern {ret} {symbol}({params});\n",
            ret = c_return_type(module, signature)?,
            symbol = function
                .artifacts
                .aot
                .as_ref()
                .ok_or_else(|| AotError(format!("missing AOT artifact for `{}`", function.name)))?
                .symbol,
            params = c_param_list(module, signature, true)?,
        );
        source.push_str(&decl);
    }
    source.push_str("\n");

    for bridge in &runtime_bridges {
        source.push_str(&generate_bridge_function(module, bridge)?);
    }

    for function in &native_functions {
        let symbol = function
            .artifacts
            .aot
            .as_ref()
            .ok_or_else(|| AotError(format!("missing AOT artifact for `{}`", function.name)))?
            .symbol
            .clone();
        source.push_str(&generate_native_wrapper(
            module,
            function.name.as_str(),
            &symbol,
            &function.signature,
        )?);
    }

    source.push_str("static void register_native_functions(KiraVm* vm) {\n");
    for function in &native_functions {
        source.push_str(&format!(
            "    kira_vm_register_native(vm, \"{}\", wrap_{});\n",
            function.name,
            mangle_ident(&function.name)
        ));
    }
    source.push_str("}\n\n");

    source.push_str("static void abort_with_error(const char* context, KiraError* err) {\n");
    source.push_str("    const char* msg = err && err->message ? err->message : \"unknown error\";\n");
    source.push_str("    fprintf(stderr, \"%s: %s\\n\", context, msg);\n");
    source.push_str("    if (err) { kira_error_free(err); }\n");
    source.push_str("    abort();\n");
    source.push_str("}\n\n");

    source.push_str("static char* kira_strdup(const char* src) {\n");
    source.push_str("    if (!src) { return NULL; }\n");
    source.push_str("    size_t len = strlen(src);\n");
    source.push_str("    char* out = (char*)malloc(len + 1);\n");
    source.push_str("    if (!out) { return NULL; }\n");
    source.push_str("    memcpy(out, src, len + 1);\n");
    source.push_str("    return out;\n");
    source.push_str("}\n\n");

    source.push_str("static char* kira_executable_path(void) {\n");
    source.push_str("#ifdef _WIN32\n");
    source.push_str("    char buffer[MAX_PATH];\n");
    source.push_str("    DWORD len = GetModuleFileNameA(NULL, buffer, MAX_PATH);\n");
    source.push_str("    if (len == 0 || len == MAX_PATH) { return NULL; }\n");
    source.push_str("    return kira_strdup(buffer);\n");
    source.push_str("#elif defined(__APPLE__)\n");
    source.push_str("    uint32_t size = 0;\n");
    source.push_str("    _NSGetExecutablePath(NULL, &size);\n");
    source.push_str("    if (size == 0) { return NULL; }\n");
    source.push_str("    char* buffer = (char*)malloc(size);\n");
    source.push_str("    if (!buffer) { return NULL; }\n");
    source.push_str("    if (_NSGetExecutablePath(buffer, &size) != 0) { free(buffer); return NULL; }\n");
    source.push_str("    return buffer;\n");
    source.push_str("#else\n");
    source.push_str("    char buffer[PATH_MAX];\n");
    source.push_str("    ssize_t len = readlink(\"/proc/self/exe\", buffer, sizeof(buffer) - 1);\n");
    source.push_str("    if (len <= 0) { return NULL; }\n");
    source.push_str("    buffer[len] = '\\0';\n");
    source.push_str("    return kira_strdup(buffer);\n");
    source.push_str("#endif\n");
    source.push_str("}\n\n");

    source.push_str("static char* kira_module_path(void) {\n");
    source.push_str("#ifdef KIRA_MODULE_PATH\n");
    source.push_str("    return kira_strdup(KIRA_MODULE_PATH);\n");
    source.push_str("#endif\n");
    source.push_str("    const char* env = getenv(\"KIRA_MODULE_PATH\");\n");
    source.push_str("    if (env && env[0] != '\\0') { return kira_strdup(env); }\n");
    source.push_str("    char* exe = kira_executable_path();\n");
    source.push_str("    if (!exe) { return kira_strdup(KIRA_MODULE_FILENAME); }\n");
    source.push_str("    const char* last_sep = strrchr(exe, '/');\n");
    source.push_str("#ifdef _WIN32\n");
    source.push_str("    const char* last_back = strrchr(exe, '\\\\');\n");
    source.push_str("    if (!last_sep || (last_back && last_back > last_sep)) { last_sep = last_back; }\n");
    source.push_str("#endif\n");
    source.push_str("    size_t dir_len = last_sep ? (size_t)(last_sep - exe) : 0;\n");
    source.push_str("    size_t file_len = strlen(KIRA_MODULE_FILENAME);\n");
    source.push_str("    size_t total = dir_len + (dir_len ? 1 : 0) + file_len + 1;\n");
    source.push_str("    char* path = (char*)malloc(total);\n");
    source.push_str("    if (!path) { free(exe); return NULL; }\n");
    source.push_str("    if (dir_len) { memcpy(path, exe, dir_len); }\n");
    source.push_str("    size_t pos = dir_len;\n");
    source.push_str("#ifdef _WIN32\n");
    source.push_str("    const char sep = '\\\\';\n");
    source.push_str("#else\n");
    source.push_str("    const char sep = '/';\n");
    source.push_str("#endif\n");
    source.push_str("    if (dir_len) { path[pos++] = sep; }\n");
    source.push_str("    memcpy(path + pos, KIRA_MODULE_FILENAME, file_len);\n");
    source.push_str("    path[pos + file_len] = '\\0';\n");
    source.push_str("    free(exe);\n");
    source.push_str("    return path;\n");
    source.push_str("}\n\n");

    source.push_str("static bool kira_read_module_bytes(unsigned char** out_bytes, size_t* out_len, KiraError* err) {\n");
    source.push_str("    if (!out_bytes || !out_len) { kira_error_set(err, \"module output pointers are null\"); return false; }\n");
    source.push_str("    char* path = kira_module_path();\n");
    source.push_str("    if (!path) { kira_error_set(err, \"failed to resolve module path\"); return false; }\n");
    source.push_str("    FILE* file = fopen(path, \"rb\");\n");
    source.push_str("    if (!file) { kira_error_set(err, \"failed to open compiled_module.bin\"); free(path); return false; }\n");
    source.push_str("    if (fseek(file, 0, SEEK_END) != 0) { kira_error_set(err, \"failed to seek module file\"); fclose(file); free(path); return false; }\n");
    source.push_str("    long size = ftell(file);\n");
    source.push_str("    if (size <= 0) { kira_error_set(err, \"module file is empty\"); fclose(file); free(path); return false; }\n");
    source.push_str("    rewind(file);\n");
    source.push_str("    unsigned char* buffer = (unsigned char*)malloc((size_t)size);\n");
    source.push_str("    if (!buffer) { kira_error_set(err, \"failed to allocate module buffer\"); fclose(file); free(path); return false; }\n");
    source.push_str("    size_t read = fread(buffer, 1, (size_t)size, file);\n");
    source.push_str("    fclose(file);\n");
    source.push_str("    free(path);\n");
    source.push_str("    if (read != (size_t)size) { free(buffer); kira_error_set(err, \"failed to read module file\"); return false; }\n");
    source.push_str("    *out_bytes = buffer;\n");
    source.push_str("    *out_len = (size_t)size;\n");
    source.push_str("    return true;\n");
    source.push_str("}\n\n");

    source.push_str("int main(void) {\n");
    source.push_str("    KiraError err = {0};\n");
    source.push_str("    unsigned char* module_bytes = NULL;\n");
    source.push_str("    size_t module_len = 0;\n");
    source.push_str("    if (!kira_read_module_bytes(&module_bytes, &module_len, &err)) { abort_with_error(\"failed to read module\", &err); }\n");
    source.push_str("    KiraModule* module = kira_module_from_bytes(module_bytes, module_len, &err);\n");
    source.push_str("    free(module_bytes);\n");
    source.push_str("    if (!module) { abort_with_error(\"failed to load module\", &err); }\n");
    source.push_str("    KiraVm* vm = kira_vm_new();\n");
    source.push_str("    if (!vm) { abort_with_error(\"failed to create vm\", &err); }\n");
    source.push_str("    if (!kira_vm_prepare(vm, module, &err)) { abort_with_error(\"failed to prepare vm\", &err); }\n");
    source.push_str("    register_native_functions(vm);\n");
    source.push_str(&format!(
        "    if (!kira_vm_run_entry(vm, module, \"{}\", &err)) {{ abort_with_error(\"runtime error\", &err); }}\n",
        entry_symbol
    ));
    source.push_str("    kira_vm_print_output(vm);\n");
    source.push_str("    kira_vm_free(vm);\n");
    source.push_str("    kira_module_free(module);\n");
    source.push_str("    return 0;\n");
    source.push_str("}\n");

    Ok(source)
}
