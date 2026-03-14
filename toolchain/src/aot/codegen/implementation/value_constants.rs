// Constant value operations

use inkwell::values::{BasicValueEnum, PointerValue};

use crate::runtime::type_system::TypeId;
use crate::runtime::Value;

use super::super::super::error::AotError;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn llvm_const(&mut self, value: &Value) -> Result<(TypeId, BasicValueEnum<'ctx>), AotError> {
        Ok(match value {
            Value::Bool(b) => (self.compiled.types.bool(), self.context.bool_type().const_int(*b as u64, false).into()),
            Value::Int(i) => (self.compiled.types.int(), self.context.i64_type().const_int(*i as u64, true).into()),
            Value::Float(f) => (self.compiled.types.float(), self.context.f64_type().const_float(f.0).into()),
            Value::String(s) => {
                let handle = self.const_string_handle(s)?;
                (
                    self.compiled
                        .types
                        .resolve_named("string")
                        .ok_or_else(|| AotError("missing string type".to_string()))?,
                    handle.into(),
                )
            }
            Value::Unit => {
                return Err(AotError(
                    "unit constants are not supported as stack values in AOT".to_string(),
                ))
            }
            Value::Array(_) | Value::Struct(_) => {
                return Err(AotError("aggregate constants are not supported in AOT".to_string()))
            }
        })
    }

    fn const_string_handle(&mut self, value: &str) -> Result<PointerValue<'ctx>, AotError> {
        let global = self
            .builder
            .build_global_string_ptr(value, "kira_str")
            .map_err(|e| AotError(e.to_string()))?;
        let bytes_ptr = global.as_pointer_value();
        let len = self
            .ptr_sized_int_type()
            .const_int(value.as_bytes().len() as u64, false);
        let make = self.declare_runtime_make_string();
        let call_site = self
            .builder
            .build_call(make, &[bytes_ptr.into(), len.into()], "make_string")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing string handle result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(super) fn const_usize_path(
        &mut self,
        path: &[usize],
        name: &str,
    ) -> Result<(PointerValue<'ctx>, inkwell::values::IntValue<'ctx>), AotError> {
        let usize_ty = self.ptr_sized_int_type();
        let elements = path
            .iter()
            .map(|value| usize_ty.const_int(*value as u64, false))
            .collect::<Vec<_>>();
        let array = usize_ty.const_array(&elements);
        let global = self.module.add_global(array.get_type(), None, name);
        global.set_initializer(&array);
        global.set_constant(true);
        let ptr = global.as_pointer_value();
        let len = usize_ty.const_int(path.len() as u64, false);
        Ok((ptr, len))
    }
}
