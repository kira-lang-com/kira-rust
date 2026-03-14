// Library export functionality (C ABI, headers, archives)

mod archive;
mod codegen;
mod header;

pub use archive::build_native_archive;
pub use codegen::{CAbiCodegen, ExportSpec};
pub use header::{generate_c_header, ExportedApi};
