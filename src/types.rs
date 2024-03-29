use crate::{BranchTableReader, InstructionReader};
use std::ops::Range;
use crate::validators::code::CodeValidationError;
use std::iter::empty;

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct TypeIndex(pub(crate) u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd)]
pub struct FuncIndex(pub(crate) u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd)]
pub struct TableIndex(pub(crate) u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd)]
pub struct MemoryIndex(pub(crate) u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd)]
pub struct GlobalIndex(pub(crate) u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LocalIndex(pub(crate) u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LabelIndex(pub(crate) u32);

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Eq, PartialEq, Debug)]
pub struct FunctionType {
    pub(crate) params: Box<[ValueType]>,
    pub(crate) results: Box<[ValueType]>,
}

impl FunctionType {
    fn params(&self) -> FunctionTypeParams {
        FunctionTypeParams { function_type: self, range: 0..self.params.len() }
    }

    fn results(&self) -> FunctionTypeResults {
        FunctionTypeResults { function_type: self, range: 0..self.results.len() }
    }
}

pub struct FunctionTypeParams<'a> {
    function_type: &'a FunctionType,
    range: Range<usize>,
}

impl Iterator for FunctionTypeParams<'_> {
    type Item = ValueType;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| *self.function_type.params.get(i).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl DoubleEndedIterator for FunctionTypeParams<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().map(|i| *self.function_type.params.get(i).unwrap())
    }
}

pub struct FunctionTypeResults<'a> {
    function_type: &'a FunctionType,
    range: Range<usize>,
}

impl Iterator for FunctionTypeResults<'_> {
    type Item = ValueType;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| *self.function_type.results.get(i).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl DoubleEndedIterator for FunctionTypeResults<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().map(|i| *self.function_type.results.get(i).unwrap())
    }
}

#[derive(Debug)]
pub enum ImportDescriptor {
    Func { type_index: TypeIndex },
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

#[derive(Debug)]
pub struct Import<'a> {
    pub module_name: &'a str,
    pub name: &'a str,
    pub import_descriptor: ImportDescriptor
}

#[derive(Debug)]
pub enum ExportDescriptor {
    Func { func_index: FuncIndex },
    Table { table_index: TableIndex },
    Memory { memory_index: MemoryIndex },
    Global { global_index: GlobalIndex },
}

#[derive(Debug)]
pub struct Export<'a> {
    pub(crate) name: &'a str,
    pub(crate) export_descriptor: ExportDescriptor
}

#[derive(Debug)]
pub struct Limits {
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

#[derive(Debug)]
pub struct TableType {
    pub(crate) limits: Limits
}

#[derive(Debug)]
pub struct MemoryType {
    pub(crate) limits: Limits
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalType {
    pub(crate) var_type: ValueType,
    pub(crate) mutable: bool,
}

#[derive(Debug)]
pub struct GlobalSegment<'a> {
    pub global_type: GlobalType,
    pub instruction_reader: InstructionReader<'a>,
}

#[derive(Debug)]
pub struct ElementSegment<'a> {
    pub table_index: TableIndex,
    pub instruction_reader: InstructionReader<'a>,
    pub function_indices: Box<[FuncIndex]>,
}

#[derive(Debug)]
pub struct DataSegment<'a> {
    pub memory_index: MemoryIndex,
    pub instruction_reader: InstructionReader<'a>,
    pub bytes: &'a [u8],
}

#[derive(Debug)]
pub struct Locals {
    pub count: u32,
    pub value_type: ValueType,
}

pub enum Choice<A, B> {
    EitherA(A),
    OrB(B),
}

impl<A, B> Iterator for Choice<A, B>
    where
        A: Iterator,
        B: Iterator<Item = A::Item>,
{
    type Item = A::Item;
    fn next(&mut self) -> Option<A::Item> {
        match self {
            Choice::EitherA(a) => a.next(),
            Choice::OrB(b) => b.next(),
        }
    }
}

impl<A, B> DoubleEndedIterator for Choice<A, B>
    where
        A: DoubleEndedIterator,
        B: DoubleEndedIterator<Item = A::Item>,
{
    fn next_back(&mut self) -> Option<A::Item> {
        match self {
            Choice::EitherA(a) => a.next_back(),
            Choice::OrB(b) => b.next_back(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
    TypeIndex(TypeIndex),
}

impl BlockType {
    pub(crate) fn params<'a>(&self, function_types: &'a [FunctionType]) -> Result<impl DoubleEndedIterator<Item = ValueType> + 'a, CodeValidationError> {
        return Ok(match self {
            BlockType::TypeIndex(type_index) => {
                let ty = if let Some(function_type) = function_types.get(type_index.0 as usize) {
                    function_type
                } else {
                    return Err(CodeValidationError::InvalidTypeIndex(*type_index));
                };
                Choice::EitherA(ty.params())
            }
            _ => { Choice::OrB(empty()) }
        })
    }

    pub(crate) fn results<'a>(&self, function_types: &'a [FunctionType]) -> Result<impl DoubleEndedIterator<Item = ValueType> + 'a, CodeValidationError> {
        return Ok(match self {
            BlockType::Empty => { Choice::OrB(None.into_iter()) }
            BlockType::ValueType(ty) => { Choice::OrB(Some(*ty).into_iter()) }
            BlockType::TypeIndex(type_index) => {
                let ty = if let Some(function_type) = function_types.get(type_index.0 as usize) {
                    function_type
                } else {
                    return Err(CodeValidationError::InvalidTypeIndex(*type_index));
                };
                Choice::EitherA(ty.results())
            }
        })
    }
}

#[derive(Debug)]
pub struct MemoryArgument {
    pub alignment: u32,
    pub offset: u32,
}

#[derive(Debug)]
pub enum Instruction<'a> {
    Unreachable,
    Nop,
    Block { block_type: BlockType },
    Loop { block_type: BlockType },
    If { block_type: BlockType },
    Else,
    End,
    Branch { label_index: LabelIndex },
    BranchIf { label_index: LabelIndex },
    BranchTable { branch_table_reader: BranchTableReader<'a> },
    Return,
    Call { func_index: FuncIndex },
    CallIndirect { type_index: TypeIndex },

    Drop,
    Select,

    LocalGet { local_index: LocalIndex },
    LocalSet { local_index: LocalIndex },
    LocalTee { local_index: LocalIndex },
    GlobalGet { global_index: GlobalIndex },
    GlobalSet { global_index: GlobalIndex },

    I32Load { memory_argument: MemoryArgument },
    I64Load { memory_argument: MemoryArgument },
    F32Load { memory_argument: MemoryArgument },
    F64Load { memory_argument: MemoryArgument },
    I32Load8s { memory_argument: MemoryArgument },
    I32Load8u { memory_argument: MemoryArgument },
    I32Load16s { memory_argument: MemoryArgument },
    I32Load16u { memory_argument: MemoryArgument },
    I64Load8s { memory_argument: MemoryArgument },
    I64Load8u { memory_argument: MemoryArgument },
    I64Load16s { memory_argument: MemoryArgument },
    I64Load16u { memory_argument: MemoryArgument },
    I64Load32s { memory_argument: MemoryArgument },
    I64Load32u { memory_argument: MemoryArgument },
    I32Store { memory_argument: MemoryArgument },
    I64Store { memory_argument: MemoryArgument },
    F32Store { memory_argument: MemoryArgument },
    F64Store { memory_argument: MemoryArgument },
    I32Store8 { memory_argument: MemoryArgument },
    I32Store16 { memory_argument: MemoryArgument },
    I64Store8 { memory_argument: MemoryArgument },
    I64Store16 { memory_argument: MemoryArgument },
    I64Store32 { memory_argument: MemoryArgument },
    MemorySize,
    MemoryGrow,

    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    I32Eqz,
    I32Eq,
    I32Ne,
    I32Lts,
    I32Ltu,
    I32Gts,
    I32Gtu,
    I32Les,
    I32Leu,
    I32Ges,
    I32Geu,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64Lts,
    I64Ltu,
    I64Gts,
    I64Gtu,
    I64Les,
    I64Leu,
    I64Ges,
    I64Geu,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32Divs,
    I32Divu,
    I32Rems,
    I32Remu,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32Shrs,
    I32Shru,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64Divs,
    I64Divu,
    I64Rems,
    I64Remu,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64Shrs,
    I64Shru,
    I64Rotl,
    I64Rotr,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    I32WrapI64,
    I32TruncF32s,
    I32TruncF32u,
    I32TruncF64s,
    I32TruncF64u,
    I64ExtendI32s,
    I64ExtendI32u,
    I64TruncF32s,
    I64TruncF32u,
    I64TruncF64s,
    I64TruncF64u,
    F32ConvertI32s,
    F32ConvertI32u,
    F32ConvertI64s,
    F32ConvertI64u,
    F32DemoteF64,
    F64ConvertI32s,
    F64ConvertI32u,
    F64ConvertI64s,
    F64ConvertI64u,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    I32Extend8s,
    I32Extend16s,
    I64Extend8s,
    I64Extend16s,
    I64Extend32s,

    I32TruncSatF32s,
    I32TruncSatF32u,
    I32TruncSatF64s,
    I32TruncSatF64u,
    I64TruncSatF32s,
    I64TruncSatF32u,
    I64TruncSatF64s,
    I64TruncSatF64u,
}
