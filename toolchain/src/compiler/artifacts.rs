use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ast::syntax::{ExecutionMode, PlatformsMetadata};
use crate::runtime::{
    type_system::{TypeId, TypeSystem},
    Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendKind {
    Vm,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    LoadConst(usize),
    LoadLocal(usize),
    StoreLocal(usize),
    Negate,
    CastIntToFloat,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Less,
    Greater,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    BuildArray { type_id: TypeId, element_count: usize },
    BuildStruct { type_id: TypeId, field_count: usize },
    ArrayLength,
    ArrayIndex,
    StructField(usize),
    StoreLocalField { local: usize, path: Vec<usize> },
    ArrayAppendLocal(usize),
    JumpIfFalse(usize),
    Jump(usize),
    Call { function: String, arg_count: usize },
    Pop,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub local_count: usize,
    pub local_types: Vec<TypeId>,
}

impl Chunk {
    pub(crate) fn push_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub params: Vec<TypeId>,
    pub return_type: TypeId,
    pub function_type: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinFunction {
    pub name: String,
    pub signature: FunctionSignature,
    pub backend: BackendKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FfiLink {
    pub library: String,
    pub header: String,
    pub search_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FfiFunction {
    pub symbol: String,
    pub signature: FunctionSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FfiMetadata {
    pub links: Vec<FfiLink>,
    pub functions: HashMap<String, FfiFunction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStage {
    BuildTimeOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotArtifact {
    pub symbol: String,
    pub target_platforms: Vec<String>,
    pub stage: BuildStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotJob {
    pub function: String,
    pub artifact: AotArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotBuildPlan {
    pub stage: BuildStage,
    pub jobs: Vec<AotJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FunctionArtifacts {
    pub bytecode: Option<Chunk>,
    pub aot: Option<AotArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledFunction {
    pub name: String,
    pub declared_mode: ExecutionMode,
    pub target_platforms: Vec<String>,
    pub selected_backend: BackendKind,
    pub signature: FunctionSignature,
    pub artifacts: FunctionArtifacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledModule {
    pub platforms: Option<PlatformsMetadata>,
    pub aot_plan: AotBuildPlan,
    pub types: TypeSystem,
    pub builtins: HashMap<String, BuiltinFunction>,
    pub ffi: FfiMetadata,
    pub functions: HashMap<String, CompiledFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompileError {}
