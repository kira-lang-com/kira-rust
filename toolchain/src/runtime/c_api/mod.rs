// C API surface for the runtime

pub mod error;
pub mod module;
pub mod value;
pub mod vm;

pub use error::KiraError;
pub use module::KiraModule;
pub use value::KiraValue;

pub type KiraNativeHandler = extern "C" fn(
    *mut vm::KiraVm,
    *const KiraModule,
    *const *const KiraValue,
    usize,
    *mut KiraError,
) -> *mut KiraValue;
