//! Type Mapping for LLVM IR Generation
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 3
//! Implements SPEC.md 12.1 Type Mapping

use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, PointerType};

pub struct TypeMapper<'ctx> {
    pub context: &'ctx Context,
}

impl<'ctx> TypeMapper<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    pub fn map_to_basic(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::Int | Type::Int64 | Type::UInt | Type::UInt64 | Type::Pointer(_) => {
                self.context.i64_type().into()
            },
            Type::Int8 | Type::UInt8 => self.context.i8_type().into(),
            Type::Int16 | Type::UInt16 => self.context.i16_type().into(),
            Type::Int32 | Type::UInt32 => self.context.i32_type().into(),
            Type::Float | Type::Float32 => self.context.f32_type().into(),
            Type::Float64 => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),
            Type::Char => self.context.i8_type().into(),
            Type::String => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            Type::Unit => self.context.i64_type().into(),
            Type::Array(elem_ty, _size) => {
                let llvm_elem = self.map_to_basic(elem_ty);
                let arr = llvm_elem.into_array_type();
                arr.into()
            },
            Type::Tuple(_types) => self.context.opaque_struct_type("tuple").into(),
            Type::Struct => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        }
    }

    pub fn map_to_pointer(&self, _ty: &Type) -> PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
    }

    pub fn get_type_size(&self, ty: &Type) -> u64 {
        match ty {
            Type::Int | Type::Int64 | Type::UInt | Type::UInt64 | Type::Pointer(_) => 8,
            Type::Int8 | Type::UInt8 | Type::Char | Type::Bool => 1,
            Type::Int16 | Type::UInt16 => 2,
            Type::Int32 | Type::UInt32 | Type::Float => 4,
            Type::Float32 => 4,
            Type::Float64 => 8,
            Type::String => 8,
            Type::Unit => 0,
            Type::Array(elem_ty, size) => self.get_type_size(elem_ty) * *size as u64,
            Type::Tuple(types) => types.iter().map(|t| self.get_type_size(t)).sum(),
            Type::Struct => 8,
        }
    }

    pub fn map_to_return_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::Unit => self.context.i64_type().into(),
            _ => self.map_to_basic(ty),
        }
    }

    pub fn fits_in_register(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::Int
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float
                | Type::Float32
                | Type::Float64
                | Type::Bool
                | Type::Char
                | Type::Pointer(_)
        )
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Float32,
    Float64,
    Bool,
    Char,
    String,
    Unit,
    Array(Box<Type>, usize),
    Tuple(Vec<Type>),
    Struct,
    Pointer(Box<Type>),
}
