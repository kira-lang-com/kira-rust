// Runtime function declarations and calls

use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;

use crate::runtime::type_system::TypeId;

use super::super::super::error::AotError;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn declare_runtime_box_int(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_int",
            handle.fn_type(&[self.context.i64_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_box_bool(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_bool",
            handle.fn_type(&[self.context.bool_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_box_float(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_float",
            handle.fn_type(&[self.context.f64_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_unbox_int(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_int",
            self.context.i64_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_unbox_bool(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_bool",
            self.context.bool_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_unbox_float(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_float",
            self.context.f64_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_clone_value(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_clone_value",
            handle.fn_type(&[handle.into()], false),
        )
    }

    pub(super) fn declare_runtime_make_string(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_make_string",
            handle.fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                ],
                false,
            ),
        )
    }

    pub(super) fn declare_runtime_print_int(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_int",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.i64_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_print_bool(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_bool",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.bool_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_print_float(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_float",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.f64_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_print_value(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        let handle = self.value_handle_type();
        self.declare_runtime_function(
            "kira_native_print_value",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), handle.into()], false),
        )
    }

    pub(super) fn declare_runtime_value_eq(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_value_eq",
            self.context.bool_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_value_eq(
        &mut self,
        left: PointerValue<'ctx>,
        right: PointerValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        let f = self.declare_runtime_value_eq();
        let call_site = self
            .builder
            .build_call(f, &[left.into(), right.into()], "value_eq")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing eq result".to_string()))
            .map(|v| v.into_int_value())
    }

    pub(super) fn declare_runtime_new_array(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_new_array",
            handle.fn_type(&[], false),
        )
    }

    pub(super) fn call_runtime_new_array(&mut self) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_new_array();
        let call_site = self
            .builder
            .build_call(f, &[], "new_array")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing new array result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(super) fn declare_runtime_array_push(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_push",
            self.context.void_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_array_push(
        &mut self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_array_push();
        self.builder
            .build_call(f, &[array.into(), value.into()], "push")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(super) fn declare_runtime_array_append(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_append",
            self.context.void_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_array_append(
        &mut self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_array_append();
        self.builder
            .build_call(f, &[array.into(), value.into()], "append")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(super) fn declare_runtime_array_length(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_length",
            self.context
                .i64_type()
                .fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn call_runtime_array_length(
        &mut self,
        array: PointerValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        let f = self.declare_runtime_array_length();
        let call_site = self
            .builder
            .build_call(f, &[array.into()], "len")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing array length result".to_string()))
            .map(|v| v.into_int_value())
    }

    pub(super) fn declare_runtime_array_index(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_array_index",
            handle.fn_type(
                &[handle.into(), self.context.i64_type().into()],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_array_index(
        &mut self,
        array: PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_array_index();
        let call_site = self
            .builder
            .build_call(f, &[array.into(), index.into()], "idx")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing array index result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(super) fn declare_runtime_new_struct(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_new_struct",
            handle.fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                ],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_new_struct(
        &mut self,
        ctx: PointerValue<'ctx>,
        type_id: TypeId,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_new_struct();
        let usize_ty = self.ptr_sized_int_type();
        let type_id_value = usize_ty.const_int(type_id.0 as u64, false);
        let call_site = self
            .builder
            .build_call(f, &[ctx.into(), type_id_value.into()], "new_struct")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing new struct result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(super) fn declare_runtime_struct_set_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        self.declare_runtime_function(
            "kira_native_struct_set_field",
            self.context.void_type().fn_type(
                &[
                    self.value_handle_type().into(),
                    usize_ty.into(),
                    self.value_handle_type().into(),
                ],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_struct_set_field(
        &mut self,
        target: PointerValue<'ctx>,
        field_index: usize,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_struct_set_field();
        let usize_ty = self.ptr_sized_int_type();
        let index = usize_ty.const_int(field_index as u64, false);
        self.builder
            .build_call(f, &[target.into(), index.into(), value.into()], "set_field")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(super) fn declare_runtime_struct_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_struct_field",
            handle.fn_type(
                &[handle.into(), usize_ty.into()],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_struct_field(
        &mut self,
        target: PointerValue<'ctx>,
        field_index: usize,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_struct_field();
        let usize_ty = self.ptr_sized_int_type();
        let index = usize_ty.const_int(field_index as u64, false);
        let call_site = self
            .builder
            .build_call(f, &[target.into(), index.into()], "get_field")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing struct field result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(super) fn declare_runtime_store_struct_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        self.declare_runtime_function(
            "kira_native_store_struct_field",
            self.context.void_type().fn_type(
                &[
                    self.value_handle_type().into(),
                    usize_ty.ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                    self.value_handle_type().into(),
                ],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_store_struct_field(
        &mut self,
        target: PointerValue<'ctx>,
        path: PointerValue<'ctx>,
        len: inkwell::values::IntValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_store_struct_field();
        self.builder
            .build_call(f, &[target.into(), path.into(), len.into(), value.into()], "store_field")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(super) fn declare_runtime_function(
        &self,
        name: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module
            .get_function(name)
            .unwrap_or_else(|| self.module.add_function(name, fn_type, None))
    }
}
